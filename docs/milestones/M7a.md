# M7a: routing

Goal:
- Vehicles get routes from A* over a link graph with travel-time costs.
- Link speeds come from an EMA of observed traffic.
- ALT landmarks make A* fast enough for city-scale demand.
- A vehicle whose route breaks is rerouted instead of despawned.

Demand, spawn queues, stats and region mode are M7b.

This implements `docs/plan.md` §3.5 "Routing" and the reroute budget item 1. Every value below is fixed.

## Constants

```rust
pub const LEFT_TURN_PENALTY: f64 = 5.0;
pub const UTURN_PENALTY: f64 = 10.0;
pub const HEURISTIC_SPEED: f64 = 27.8;
pub const EMA_REFRESH_TICKS: u64 = 600;
pub const EMA_ALPHA: f64 = 0.3;
pub const MIN_LINK_SPEED: f64 = 1.0;
pub const LANDMARK_COUNT: usize = 8;
```

## `src/routing/`

### `graph.rs`: `RouteGraph`

A CSR successor table over link ids (`0 .. 2 × road_count`), built from the `Network`:
- `succ_start: Vec<u32>` has length `link_count + 1`, and `succ: Vec<Succ>`, where `Succ { to: LinkId, penalty: f32 }`.
- For every active link `a`, the successors are the movements at `link_to(a)`, in `(from_link, to_link)` order, taken from `ensure_junction`.
  - The penalty is `LEFT_TURN_PENALTY` for `Left`, `UTURN_PENALTY` for `UTurn`, and 0 otherwise.
- `pred_start`/`pred` form the reverse CSR, used for backward Dijkstra.
- `RouteGraph::build(network: &mut Network) -> RouteGraph` builds every junction once. It records `network.version()` in `built_version`.
- `Sim` rebuilds the graph at the start of a step whenever `network.version() != graph.built_version`.

### `costs.rs`: `LinkCosts`

- `free_speed: Vec<f64>` is the speed limit of each link's road.
- `ema_speed: Vec<f64>` starts equal to `free_speed`.
- `drive_len: Vec<f64>` is `link_span.1 − link_span.0`, at least 0.1.
- `travel_time(link) = drive_len / max(ema_speed, MIN_LINK_SPEED)`.
- `free_time(link) = drive_len / free_speed`.
- **`refresh(&mut self, samples: &[(LinkId, f64)])`:**
  - `samples` holds each link-placed vehicle's `(link, v)`, in slot order.
  - Per link: `mean = sum / count`, then `ema = (1 − α)·ema + α·min(mean, free_speed)`.
  - A link with no samples uses `ema = (1 − α)·ema + α·free_speed`.
  - Accumulate with a `Vec<(f64, u32)>` indexed by link, then iterate links in id order.
- `Sim` calls `refresh` when `tick % EMA_REFRESH_TICKS == 0 && tick > 0`, after integration.
- The EMA never exceeds `free_speed`, so free-flow times are lower bounds and the ALT bounds stay admissible.

### `landmarks.rs`: `Landmarks` (ALT)

- **Selection:**
  1. Run Dijkstra forward from link 0 on free-flow costs, where edge cost is `free_time(b) + penalty`.
  2. Landmark 1 is the reachable link with the largest distance. Ties go to the lowest id.
  3. Each next landmark is the link maximising the minimum forward distance from the landmarks chosen so far, over reachable links. Ties go to the lowest id.
  4. Stop at `LANDMARK_COUNT`, or earlier if no reachable link remains.
- **Tables** for each landmark `L`:
  - `from_l[L][v]`: forward Dijkstra from `L`.
  - `to_l[L][v]`: backward Dijkstra to `L` over `pred`.
  - Both are stored as `f32`, with unreachable as `f32::INFINITY`.
- **Heuristic** `h(v, t)`:
  - `max(euclid(v, t) / HEURISTIC_SPEED, max over L of (from_l[L][t] − from_l[L][v]), max over L of (to_l[L][v] − to_l[L][t]))`.
  - Terms with an infinity are skipped, and the result is at least 0.
  - `euclid` measures from the to-node of `v` to the from-node of `t`.
- **Timing:** built once in `Sim::new`. Edits in M10 keep these tables, which may make bounds loose or slightly inadmissible. That is accepted: routes stay valid, just possibly suboptimal.
- **Dijkstra:** uses the same heap type as A*, with an epoch-stamped visited array.

### `astar.rs`: `Router`

```rust
pub struct Router { g: Vec<f64>, parent: Vec<u32>, stamp: Vec<u32>, epoch: u32, heap: BinaryHeap<HeapItem> }
pub struct RouteStats { pub queries: u64, pub expanded: u64, pub nanos: u64 }
```

- **`route(&mut self, graph, costs, landmarks, network, from: LinkId, to: LinkId) -> Option<Vec<LinkId>>`:**
  - Edge cost `a → b` is `travel_time(b) + penalty`. The start link costs 0.
  - `f = g + h`.
  - **Heap order:** `(f.to_bits() ascending, link ascending)`. Every `f ≥ 0`, so bit order equals numeric order.
  - **Stale entries:** an entry whose recorded `g` is greater than the current `g[link]` is skipped.
  - **Reset:** the `stamp`/`epoch` pattern avoids clearing arrays. `epoch` wraps by resetting `stamp`.
  - **Result:** `[from, …, to]`, or `None` when the target is unreachable or inactive. `from == to` returns `[from]`.
- Timing uses no wall clock inside `sim-core`. `RouteStats` counts only queries and expansions; the bench example measures time outside.

## Reroute on invalid route (`Sim::step`)

This replaces M5's despawn:
- A vehicle whose next movement or `entry_lane` is missing reroutes from its current link to its route's last link.
- On success, its new route is appended to the arena (the old one counts toward `dead_len`), and `cursor = 0`.
- If no route exists, it despawns and increments `stranded`.
- This happens with no per-step limit, as plan budget item 1 says.
- It is checked in step 1 (the junction prefetch pass), in slot order, before any decision.

## `Sim` API additions

- `pub fn route(&mut self, from: LinkId, to: LinkId) -> Option<Vec<LinkId>>` wraps the router.
- `pub fn spawn_trip(&mut self, from: LinkId, to: LinkId) -> Result<u32, SpawnError>` routes, then spawns. It adds `SpawnError::NoRoute`.
- `pub fn route_stats(&self) -> RouteStats`, `pub fn stranded(&self) -> u64`.

## Tests

1. **`astar_optimal`:** on `grid_city(10×10 @ 150)`, for 50 random `(from, to)` pairs (seed `Pcg32::new(11, 3)`), the A* cost with ALT equals a plain Dijkstra cost with the same edge costs, to within 1e-6 relative.
   - Also compare against A* with only the euclidean term.
   - Expose `pub fn dijkstra_cost` for tests, in `routing/dijkstra.rs`, reusing the heap.
2. **`astar_respects_bans_and_oneways`:**
   - On `four_way`, ban one turn and check that the route avoids it.
   - On `one_way_pair`, no route uses a missing link direction.
3. **`unreachable`:** a `MapBuilder` map with two disconnected components. Routing from a link in one component to a link in the other returns `None`, and `spawn_trip` returns `NoRoute`.
4. **`ema_refresh`:** feed samples at half the free speed repeatedly. After 10 refreshes, `ema` is within 5 % of half. With no samples, it recovers toward `free_speed`.
5. **`landmarks_admissible`:** for 200 random pairs on `grid_city(20×20 @ 150)` with free-flow costs, `h(v, t) ≤ dijkstra_cost(v, t) + 1e-6`.
6. **`reroute_on_invalid`:**
   - Spawn a trip, then ban (via `Network` test access) the vehicle's next turn and invalidate that node.
   - After the next step, the vehicle has a new valid route and still reaches its destination.
   - Add `pub fn ban_turn_for_test` on `Network` behind the `fixtures` feature.
7. **`determinism`:** unchanged hashes still match across two runs. `FOUR_WAY_HASH_5000` may change only if spawns changed; they should not, because the M5/M6 tests spawn by explicit route.

## Example (`examples/route_bench.rs`)

- Loads a map (path argument), builds the `Sim`, and runs 2,000 random trips (seed fixed), recording each route's duration with `std::time::Instant` in the example.
- Prints: link count, landmark build ms, mean and p95 route µs, mean expansions, and the mean route length in links.
- Run it on a large synthetic grid, generated into the scratchpad with `mapgen --synthetic 120x120@150` (18 km square).

## Acceptance

- `bash scripts/check.sh` passes.
- Report the `route_bench` output for `120x120@150`. The target is a mean A* time of at most 0.5 ms native.
- No comments. Do not modify `CLAUDE.md`, `docs/` or `.claude/`.
