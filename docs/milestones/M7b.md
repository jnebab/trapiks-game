# M7b: demand, spawn queues, stats, region mode, bench

Goal:
- The sim generates its own traffic: Poisson trip arrivals, weighted origins and destinations, bounded route budgets, and per-link spawn queues with expiry.
- It measures city and region statistics.
- It runs in either city mode or region (challenge) mode.
- A bench example proves the §3.9 step targets on a large synthetic city.

This implements `docs/plan.md` §3.5 ("Demand", "Stats", "Reroute budget" items 3 and 4) and the region part of §3.7. Every value below is fixed.

## Constants

```rust
pub const SPAWN_ROUTE_BUDGET: usize = 4;
pub const TRIP_EXPIRY_TICKS: u64 = 600;
pub const DESTINATION_DRAWS: u32 = 8;
pub const CITY_TRIP_BAND: (f64, f64) = (1_000.0, 12_000.0);
pub const REGION_TRIP_BAND: (f64, f64) = (300.0, 4_000.0);
pub const BOUNDARY_WEIGHT: f64 = 5.0;
```

`RoadClass::demand_weight()`:

| Class | Weight |
|---|---|
| motorway | 8 |
| trunk | 6 |
| primary | 4 |
| secondary | 3 |
| tertiary | 2 |
| any other | 1 |

Links (for example `PrimaryLink`) use their parent class's weight.

## Configuration (`src/config.rs`, ts-rs exported)

```rust
pub enum SimMode { City, Region { center_x: f64, center_y: f64, radius: f64 } }
pub struct SimConfig { pub seed: u64, pub mode: SimMode, pub vehicles_per_hour: f64 }
```

- `Sim::from_config(map, config) -> Sim`. In region mode it calls `network.set_region` before building the route graph, the landmarks and the demand tables. `Sim::new(map, seed)` is `City` with `vehicles_per_hour = 0`.
- `Sim::set_demand(vehicles_per_hour)`.
- `#[cfg(feature = "fixtures")] pub fn set_trip_band_for_test(&mut self, band: (f64, f64))`.
- `pub fn demand_tables(&self) -> &DemandTables`.

## `src/demand/`

### `tables.rs`: `DemandTables`

`origins` and `destinations`, each as `links: Vec<LinkId>` with `cumulative: Vec<f64>`, over active links in id order:
- Origin weight: `drive_len × demand_weight`, multiplied by `BOUNDARY_WEIGHT` when the link is a region source.
- Destination weight: the same, with `BOUNDARY_WEIGHT` for region sinks.
- Zero-weight links are omitted.
- `sample(&self, rng) -> Option<LinkId>`: `x = rng.next_f64() × total`, then the first `cumulative > x`, found by binary search. `None` when empty.
- Rebuilt whenever `network.version()` changes, at the same point as the route graph.

### `trip_clock.rs`: `TripClock`

- `next_s: f64` starts at the first exponential draw: `−libm::log(1 − rng.next_f64()) / rate`, where `rate = vehicles_per_hour / 3600`.
- Each step, with `time_s = tick × DT`, create a trip while `next_s ≤ time_s`, then advance `next_s += −libm::log(1 − rng.next_f64()) / rate`.
- A rate of 0 sets `next_s = f64::INFINITY` without drawing.
- `set_rate` restarts the schedule from the current time, with one draw when the rate is above 0.

### `trips.rs`

- `Trip { from: LinkId, to: LinkId, created_tick: u64 }`.
- **Trip creation:**
  1. Sample the origin.
  2. Draw the destination up to `DESTINATION_DRAWS` times until the straight-line distance, from the origin's to-node to the destination's from-node, is inside the mode's band. A destination must differ from the origin.
  3. Every creation counts `created += 1`. If no draw lands in the band, it also counts `unserved += 1`.
- **`trip_queue: VecDeque<Trip>`**, in FIFO order.
  - Each step, drop expired trips from the front (`tick − created_tick ≥ TRIP_EXPIRY_TICKS` → `unserved += 1`).
  - Then route up to `SPAWN_ROUTE_BUDGET` trips from the front. No route gives `unserved += 1`. Otherwise the routed trip moves to its origin link's queue.
- **`LinkQueues`** is a module with `BTreeMap<LinkId, VecDeque<RoutedTrip>>`, where `RoutedTrip { trip, route: Vec<LinkId> }`.
  - It exposes `step(tick, spawn: impl FnMut(&RoutedTrip) -> Result<u32, SpawnError>) -> QueueOutcome { spawned, unserved }`, so it can be tested alone.
  - Routes are produced with `route_into` into a reused buffer, then moved into the trip.
  - Each step, in key order: drop expired trips at the front (counted as unserved), then try to spawn the front trip.
  - `Blocked` leaves it queued. Any other error drops it as unserved.
  - Empty queues are removed.

## Stats (`src/stats.rs`)

- **Per vehicle:** `free_flow: Vec<f64>`, set at spawn to `Σ free_time(link)` over the route.
  - After a reroute: `elapsed + current-place term + Σ free_time` over the new route's remaining links.
  - On a link, the current-place term is `(span_end − s) / free_speed(current link)`.
  - In a movement, it is `(length − s) / free_speed(route[cursor+1])`.
- **`StatsWindow`:** `start_tick`, `created`, `arrivals`, `travel_sum`, `delay_sum`, `accrued_delay`, `spawned`, `unserved`, `stranded`, with `reset(tick)`.
  - An **arrival** is a despawn at the end of the last link. It adds `travel = (tick − spawn_tick) × DT` and `delay = max(0, travel − free_flow)`.
  - `accrued_delay` grows every step by `Σ DT × max(0, 1 − v/v0)` over live vehicles.
  - Stranded vehicles and unserved trips are only counted here. `UNSERVED_DELAY` is applied in M12 scoring.
- **`StatsSnapshot`** (ts-rs exported): `tick`, `sim_time_s`, `active`, `created`, `spawned`, `arrivals`, `unserved`, `stranded`, `mean_travel_s`, `mean_delay_s`, `accrued_delay_s`, `throughput_per_hour`, `mean_speed`, `stopped_share`.
  - Means over zero arrivals are 0.
  - `throughput_per_hour = arrivals / window_hours`, or 0 when the window is empty (`tick == start_tick`).
  - `mean_speed` and `stopped_share` (`v < 0.5`) are instantaneous, over live vehicles.
  - `Sim::stats()` builds it. `Sim::reset_stats()` resets the window.
- **`Sim::road_speed_ratio(&self, out: &mut Vec<f32>)`:** resizes to `road_count`. Each road's value is the mean over link-placed vehicles on that road of `v / free_speed`, or 1.0 when the road is empty. Accumulate in one pass, in slot order.

## Step order after M7b

1. Rebuild the graph and demand tables if the network version changed. Landmarks are built only at `new`.
2. Prefetch junctions and reroute invalid routes (M7a).
3. Trip clock, then trip creation.
4. Route up to the budget, then fill the link queues.
5. **Rebuild the occupancy.**
6. **Spawn from the link queues.** New vehicles go into `pending`.
   - A pending vehicle is always the lowest `s` on its lane, so no existing vehicle needs it as a leader.
   - `index_of` returns `None` for it. `leader()` then uses the last entry of `range(key)` (which excludes pending), never `last_on`.
7. Zone bookkeeping, decisions, acceleration, integration and transitions (M5 and M6).
8. Stats accrual, EMA accumulation and refresh, and compaction, then `tick += 1`.

`state_hash` also covers:
- the trip clock (`next_s` bits)
- every queued trip's `(from, to, created_tick)`
- `ema_speed` bits

It does not cover the stats window. That is derived data, and `reset_stats` must not change the hash.

Update `FOUR_WAY_HASH_5000`, because the hash inputs changed.

## Tests

1. **`demand_weights`:** on a 3-road `MapBuilder` map with known lengths and classes, 100,000 samples match the weight shares within ±0.5 percentage points.
2. **`poisson_rate`:** 3,600 vph over 36,000 ticks gives `created` of 3,600 ± 5 %. Rate 0 gives none.
3. **`trip_band`:** on `four_way(2, 200.0)` in city mode, every trip is unserved because no destination is ≥ 1 km away.
4. **`queue_expiry`:** test `LinkQueues` directly.
   - Feed it `RoutedTrip`s and step it with a spawn closure that always returns `Blocked`.
   - Each trip is dropped exactly once, at `created_tick + 600`.
   - `unserved` equals the number fed.
5. **`region_mode`:** on `grid_city(20×20 @ 150)` with a region of radius 600 at the centre:
   - Only active links carry vehicles.
   - Sampling 100,000 origins from `demand_tables()` gives source links at least 3× the share they would have under `drive_len × demand_weight` without `BOUNDARY_WEIGHT`. The expected ratio is about 4×, because the skyway takes much of the mass.
   - Vehicles that reach a sink despawn as arrivals.
6. **`stats_low_demand`:** `corridor()` at 60 vph with `set_trip_band_for_test((100.0, 2_000.0))`, for 6,000 ticks.
   - `mean_delay_s < 10`.
   - `spawned − active − stranded == arrivals`.
   - `stopped_share < 0.05`.
7. **`road_speed_ratio`:** an empty map gives all 1.0. A queue at a red light gives less than 0.2 on that road.
8. **`determinism_city`:** `grid_city(40×40 @ 150)` in city mode at 20,000 vph for 3,000 ticks, run twice, gives equal hashes. Also compare the `StatsSnapshot`.

## Example (`examples/bench.rs`, replacing ad-hoc timing)

- **Arguments:** `bench <map.bin.gz> [--region x,y,r] [--vph N] [--warmup N] [--ticks N]`.
- **Reports** (step timing with `Instant` outside `sim-core`):
  - median, p95 and max step ms
  - mean and max active vehicles, and mean speed
  - `created`, `spawned`, `arrivals`, `unserved` and `stranded`
  - routes per step, and mean expansions per route from `RouteStats` deltas
  - mean route µs, from a separate timed pass of 2,000 random `sim.route` calls after the run, with the same EMA state
  - route work per step = routes per step × mean route µs
  - junctions built and landmark count
- **Runs to report:**
  1. City: `120x120@150` synthetic, `--vph 100000 --warmup 9000 --ticks 3000`. The targets are about 20k active vehicles, a median step ≤ 12 ms native, and route work ≤ 4 ms per step. Watch mean speed and `stranded`, so gridlock is visible instead of hiding behind a low step time.
  2. Region: the same map, `--region 9000,9000,2000 --vph 36000 --warmup 3000 --ticks 3000`. The targets are about 3k active vehicles and a median step ≤ 2 ms native.
- On `grid_city`, trips that start on the skyway's second half, or end on its first half, are structurally unserved: the second half has no successor and the first half no predecessor. Expect about 5 % unserved in city mode and more in regions crossing the skyway. Do not read that as gridlock.
- When a target is missed, report the numbers and the top cost from a quick profile (`perf` if available, otherwise timing splits added to the example). Don't change the spec's algorithms without the orchestrator.

## Acceptance

- `bash scripts/check.sh` passes.
- Report both bench outputs.
- No comments. Do not modify `CLAUDE.md`, `docs/` or `.claude/`.
