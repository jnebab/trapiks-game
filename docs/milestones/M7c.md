# M7c: city step performance and region landmarks

Goal: meet the plan §3.9 city targets on the large synthetic city, and fix landmark selection in region mode. Behaviour stays the same, apart from the one routing change allowed below.

**M7b measurements** (this container, `bench` on `120x120@150`):

| Run | Result | Target |
|---|---|---|
| City, 100k vph, ~23.5k active | median step 28.2 ms, p95 40.5 ms | median ≤ 12 ms |
| City route work | 2.74 routes/step × 3.8 ms = 10.4 ms/step, 6,372 expansions per route | ≤ 4 ms/step |
| Region, 2 km, 36k vph | median step 1.18 ms | ≤ 2 ms (met) |
| Region landmarks | 1 | 8 |

The implementer's M7b timing splits for the city step:

| Phase | ms |
|---|---|
| accelerations | 9.8 |
| routing new trips | 5.4 |
| decisions | 4.9 |
| zone bookkeeping | 1.8 |
| occupancy rebuild | 1.5 |
| advance | 1.0 |

## 1. Region landmarks

**Cause:** the first landmark can be a one-way sink, such as a link leaving the region. Every other link's forward distance from it is then infinite, so `farthest` finds no candidate and selection stops at one landmark.

**Fix:**
- **Candidates:** a landmark must have at least one successor and one predecessor in the route graph. This applies to landmark 1 and every later one.
- **Distance for farthest selection:** `nearest[i]` is the minimum over the chosen landmarks of `min(forward_L[i], backward_L[i])`. A link connected to a landmark in either direction therefore has a finite value.
- Ties still go to the lowest id. The seed rules are unchanged.

**Test:** `grid_city(20×20 @ 150)` in region mode (radius 600 at the centre) builds `LANDMARK_COUNT` landmarks, and `astar_optimal` still holds there.

## 2. Pure step optimizations (no behaviour change)

Before changing anything, record a **golden hash**:
- `bench` prints `state_hash` at the end of the run.
- Record it for the city run and the region run with the M7b code.

Every optimization in this section must leave both hashes, and `FOUR_WAY_HASH_5000`, unchanged.

1. **Measure.** Add temporary timing splits inside the acceleration pass (leader lookup, stop-line gap, conflict gap, IDM), the decision pass and zone bookkeeping. Report them, then remove them. `sim-core` must not keep `Instant`, because it panics on `wasm32-unknown-unknown`.
2. **Fix the top costs.** Likely candidates, to be confirmed by the splits:
   - **`conflict_gap`:** precompute per movement whether it has any non-merge conflict (`Junction.has_crossing: Vec<bool>`) and return early otherwise. Compute the vehicle's own `movement_priority` lazily, only when an entry falls in a conflict window.
   - **`RuleContext`:** build it once per pass, not per vehicle or per conflict.
   - **IDM:** `(v/v0)^4` by multiplication, not `powf`. One `sqrt`.
   - **Decisions and zone bookkeeping:** remove repeated junction and movement lookups found by the splits.
3. Keep every change a pure function of existing state. A cache must be invalidated by everything it depends on: the network version, the vehicle's place, cursor and route.

## 3. Routing under congestion (one allowed behaviour change)

**Cause:** the landmark bounds come from free-flow times, and the EMA costs under congestion are 2–3× higher. The bounds are then weak, and A* expands about 6,400 links per trip.

**Fix: weighted A* for sim routing.**
- `RouteContext` gains `heuristic_weight: f64`, and `f = g + w·h`.
- Sim routing (trips and reroutes) uses `ROUTE_HEURISTIC_WEIGHT`.
- `astar_optimal` and `route_bench` use `w = 1.0`, and `astar_optimal` must still match Dijkstra.
- **Choosing `w`:**
  - On the congested city state, after the bench warm-up, measure `w ∈ {1.0, 1.25, 1.5, 2.0}` over 2,000 random trips.
  - For each, report mean expansions, mean µs and the mean and max ratio of route cost to the Dijkstra optimum under the same costs.
  - Pick the smallest `w` that gives route work ≤ 3 ms per step with a mean cost ratio ≤ 1.05, and set `ROUTE_HEURISTIC_WEIGHT` to it.
  - If none qualifies, report the table and use 1.5.
- Add test `weighted_astar_bound`: on `grid_city(20×20 @ 150)` with randomized EMA costs, every weighted route costs at most `w ×` the Dijkstra optimum.
- Update `FOUR_WAY_HASH_5000` only if it changes. The M5 four-way scenario does not route, so it should not change.

## Acceptance

- `bash scripts/check.sh` passes.
- **Report:**
  - the golden hashes before and after section 2, which must be equal
  - the timing splits
  - the `w` table
  - three runs each of the city and region benches, with nothing else running
- **Targets:**
  - city median ≤ 12 ms, route work ≤ 4 ms/step
  - region median ≤ 2 ms, 8 landmarks
- If the city median is still above 12 ms after sections 2 and 3, stop and report the splits. Don't change behaviour beyond section 3.
- No comments. Do not modify `CLAUDE.md`, `docs/` or `.claude/`. Do not stage files.
