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
- **Turns:** add `Network::turns(&self, node) -> Vec<(LinkId, LinkId, TurnKind)>`. It is uncached and uses M3's candidate and turn-kind logic (tangents, bans, U-turn rule) without building Béziers or conflicts. Junctions stay lazy, and routing never builds them.
- **Successors:** for every active link `a`, the successors are the turns at `link_to(a)` with `from == a`, in `to` order.
  - Build by iterating nodes once: call `turns(node)` a single time per node, count first, then fill the CSR.
  - The penalty is `LEFT_TURN_PENALTY` for `Left`, `UTURN_PENALTY` for `UTurn`, and 0 otherwise.
- **Predecessors:** `pred_start`/`pred` form the reverse CSR, where `Pred { from: a, penalty }` is stored under `b`. Backward relaxation from `b` to `a` costs `free_time(b) + penalty`, where `b` is the link being popped.
- **Build:** `RouteGraph::build(network: &Network) -> RouteGraph` records `network.version()` in `built_version`, and costs O(links).
- **Staleness:** `Sim::ensure_graph()` rebuilds the graph whenever `network.version() != built_version`. It is called at the start of `step`, and by `Sim::route` and `spawn_trip`.

### `costs.rs`: `LinkCosts`

- `free_speed: Vec<f64>` is the speed limit of each link's road.
- `ema_speed: Vec<f64>` starts equal to `free_speed`.
- `drive_len: Vec<f64>` is `link_span.1 − link_span.0`, at least 0.1.
- `travel_time(link) = drive_len / max(ema_speed, MIN_LINK_SPEED)`.
- `free_time(link) = drive_len / free_speed`.
- **Speed accumulation:** every tick, `accumulate(link, v)` is called for each link-placed vehicle, in slot order, into `window: Vec<(f64, u32)>` indexed by link. Sampling every tick, instead of taking a snapshot, avoids aliasing with the 700-tick signal cycle.
- **`refresh(&mut self)`**, per link in id order:
  - `mean = sum / count`, then `ema = (1 − α)·ema + α·min(mean, free_speed)`.
  - A link with no samples uses `ema = (1 − α)·ema + α·free_speed`.
  - Then clear the window.
- `Sim` calls `refresh` when `tick % EMA_REFRESH_TICKS == 0 && tick > 0`, after integration.
- **Growth:** tables grow to the current link count, which increases after M15 `AddRoad`. A missing entry reads as `free_speed` or infinity, as appropriate, through `.get()`.
- The EMA never exceeds `free_speed`, so free-flow times are lower bounds and the ALT bounds stay admissible.

### `landmarks.rs`: `Landmarks` (ALT)

- **Selection:**
  1. Run Dijkstra forward from the lowest active link id on free-flow costs, where edge cost is `free_time(b) + penalty`.
  2. Landmark 1 is the reachable link with the largest distance. Ties go to the lowest id.
  3. Each next landmark is the link maximising the minimum forward distance from the landmarks chosen so far, over reachable links. Ties go to the lowest id.
     - The forward Dijkstra from each chosen landmark is exactly `from_l[L]`, so reuse it. That makes 1 + 8 + 8 Dijkstras in total.
  4. Stop at `LANDMARK_COUNT`, or earlier if no reachable link remains.
  5. If the seed reaches fewer than half of the active links (for example a one-way sink, or a fragment of a region), reseed from the lowest unreached active link and use the largest reachable set. Try at most 4 seeds.
- `Landmarks::count()` is exposed. The bench and the region test assert `count() == LANDMARK_COUNT` when the graph has at least that many links.
- The build time is printed by `route_bench`. Expect 2–5 s in wasm on the real map.
- **Tables** for each landmark `L`:
  - `from_l[L][v]`: forward Dijkstra from `L`.
  - `to_l[L][v]`: backward Dijkstra to `L` over `pred`.
  - Both are stored as `f32`, with unreachable as `f32::INFINITY`.
- **Heuristic** `h(v, t)`:
  - `max(euclid(v, t) / heuristic_speed, max over L of (from_l[L][t] − from_l[L][v]), max over L of (to_l[L][v] − to_l[L][t]))`.
  - Differences are computed in f64 after widening.
  - Terms with an infinity, or with an id outside the tables (`.get()` returns `None`), are skipped. The result is at least 0.
  - `euclid` measures from the to-node of `v` to the from-node of `t`.
  - `heuristic_speed = max(HEURISTIC_SPEED, max free_speed over links)`, computed at build. `drive_len` excludes setbacks, so the Euclidean term can still slightly overestimate on short fast links. Routes stay valid.
  - The target-side values (`from_l[L][t]`, `to_l[L][t]` and the target point) are hoisted out of the expansion loop.
- **Timing:** built once in `Sim::new`. Edits in M10 keep these tables, which may make bounds loose or slightly inadmissible. That is accepted: routes stay valid, just possibly suboptimal.
- **Dijkstra:** uses the same heap type as A*, with an epoch-stamped visited array.

### `astar.rs`: `Router`

```rust
pub struct Router { g: Vec<f64>, parent: Vec<u32>, stamp: Vec<u32>, epoch: u32, heap: BinaryHeap<HeapItem> }
pub struct RouteStats { pub queries: u64, pub expanded: u64 }
```

- **`route_into(&mut self, ctx: RouteContext, from: LinkId, to: LinkId, out: &mut Vec<LinkId>) -> bool`:**
  - `RouteContext { graph, costs, landmarks: Option<&Landmarks>, network }`.
  - Edge cost `a → b` is `travel_time(b) + penalty`. The start link costs 0.
  - `f = g + h`.
  - **Heap order:** `(f.to_bits() ascending, g.to_bits() descending, link ascending)`. Every `f ≥ 0`, so bit order equals numeric order. Preferring larger `g` on equal `f` avoids expanding whole tie diamonds on grids.
  - **Stale entries:** an entry whose recorded `g` is greater than the current `g[link]` is skipped.
  - **Reset:** `stamp` starts all 0 and `epoch` starts at 1. On wrap, set `stamp` to 0 and `epoch = 1`.
  - **Sizing:** `g`, `parent` and `stamp` are resized to `link_count` at the start of each query.
  - **Result:** writes `[from, …, to]` into `out` and returns true. It returns false when the target is unreachable or inactive. `from == to` gives `[from]`.
- `RouteStats { queries, expanded }` holds counts only; `sim-core` uses no wall clock. The bench times routes outside.

## Reroute on invalid route (`Sim::step`)

This replaces M5's despawn. It is checked in step 1 (the junction prefetch pass), in slot order, before any decision, with no per-step limit (plan budget item 1).

- **Trigger:** a vehicle whose next movement, or `entry_lane`, is missing.
- **Link-placed vehicle:** route from `route[cursor]` to the route's last link, store the result, and set `cursor = 0`.
- **Movement-placed vehicle:** route from `route[cursor+1]` to the last link, then store `[route[cursor], new…]` with `cursor = 0`, so the pending movement → link transition still lands correctly.
- **Arena:** the new route is appended, and the old one counts toward `dead_len`.
- **State reset:** a reroute clears M6's `committed`, `arrival_tick`, `wait_ticks` and `stopped_at_line` for the vehicle.
- **Validation:** after a reroute, verify that the first transition exists in the junction. If it does not, or if no route exists, the vehicle despawns and `stranded` is incremented. A stale graph can therefore never loop a vehicle.
- **Step 5:** M5's missing-movement branch stays only as the `let … else` fallback where an `Option` is unwrapped. It strands the vehicle and is unreachable in practice.

## `Sim` API additions

- `pub fn route(&mut self, from: LinkId, to: LinkId) -> Option<Vec<LinkId>>` calls `ensure_graph`, then wraps the router.
- `pub fn spawn_trip(&mut self, from: LinkId, to: LinkId) -> Result<u32, SpawnError>` routes, then spawns. It adds `SpawnError::NoRoute`.
- `pub fn route_stats(&self) -> RouteStats`, `pub fn stranded(&self) -> u64`.

## Tests

1. **`astar_optimal`:** on `grid_city(10×10 @ 150)`, for 50 random `(from, to)` pairs (seed `Pcg32::new(11, 3)`), the A* cost with ALT equals a plain Dijkstra cost with the same edge costs, to within 1e-6 relative.
   - Also compare against A* with `landmarks: None`, which uses the Euclidean term only.
   - Expose `pub fn dijkstra_cost` for tests, in `routing/dijkstra.rs`, reusing the heap.
2. **`astar_respects_bans_and_oneways`:**
   - On `four_way`, ban one turn and check that the route avoids it.
   - On `one_way_pair`, no route uses a missing link direction.
3. **`unreachable`:** a `MapBuilder` map with two disconnected components. Routing from a link in one component to a link in the other returns `None`, and `spawn_trip` returns `NoRoute`.
4. **`ema_refresh`:** feed samples at half the free speed repeatedly. After 10 refreshes, `ema` is within 5 % of half. With no samples, it recovers toward `free_speed`.
5. **`landmarks_admissible`:** for 200 random pairs on `grid_city(20×20 @ 150)` with free-flow costs, `h(v, t) ≤ dijkstra_cost(v, t) + 1e-3` (seconds, absolute; the tables are f32).
6. **`reroute_on_invalid`:** on `four_way(2, 200.0)`:
   - Spawn `[0, 3]`, then call `ban_turn_for_test(node 0, road 0, road 1)`.
   - `ban_turn_for_test` lives on `Network` behind the `fixtures` feature. It invalidates the node and increments `version`.
   - After the next step, the vehicle's route no longer contains the banned turn and is a valid movement chain, via a dead-end U-turn.
   - The vehicle reaches the end of link 3.
   - A second case bans every exit, so the vehicle strands and `stranded == 1`.
7. **`determinism`:** unchanged hashes still match across two runs. `FOUR_WAY_HASH_5000` may change only if spawns changed; they should not, because the M5/M6 tests spawn by explicit route.

## Example (`examples/route_bench.rs`)

- Loads a map (path argument), builds the `Sim`, and runs 2,000 random trips (seed fixed), recording each route's duration with `std::time::Instant` in the example.
- Prints: link count, landmark build ms, mean and p95 route µs, mean expansions, and the mean route length in links.
- Run it on a large synthetic grid, generated into the scratchpad with `mapgen --synthetic 120x120@150` (18 km square).

## Acceptance

- `bash scripts/check.sh` passes.
- Report the `route_bench` output for `120x120@150`. The target is a mean A* time of at most 0.5 ms native.
- No comments. Do not modify `CLAUDE.md`, `docs/` or `.claude/`.
