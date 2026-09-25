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
pub const UNSERVED_DELAY: f64 = 300.0;
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

## `src/demand/`

### `tables.rs`: `DemandTables`

`origins` and `destinations`, each as `links: Vec<LinkId>` with `cumulative: Vec<f64>`, over active links in id order:
- Origin weight: `drive_len × demand_weight`, multiplied by `BOUNDARY_WEIGHT` when the link is a region source.
- Destination weight: the same, with `BOUNDARY_WEIGHT` for region sinks.
- Zero-weight links are omitted.
- `sample(&self, rng) -> Option<LinkId>`: `x = rng.next_f64() × total`, then the first `cumulative > x`, found by binary search. `None` when empty.
- Rebuilt whenever `network.version()` changes, at the same point as the route graph.

### `arrivals.rs`: `Arrivals`

- `next_arrival_s: f64` starts at the first exponential draw, `−libm::log(1 − rng.next_f64()) / rate`.
- Each step, with `time_s = tick × DT`, create a trip while `next_arrival_s ≤ time_s`, then advance `next_arrival_s += −libm::log(1 − rng.next_f64()) / rate`, where `rate = vehicles_per_hour / 3600`.
- `rate = 0` produces nothing.
- `set_rate` restarts the schedule from the current time.

### `trips.rs`

- `Trip { from: LinkId, to: LinkId, created_tick: u64 }`.
- **Trip creation:**
  1. Sample the origin.
  2. Draw the destination up to `DESTINATION_DRAWS` times until the straight-line distance, from the origin's to-node to the destination's from-node, is inside the mode's band. A destination must differ from the origin.
  3. If no draw lands in the band, count `unserved += 1`.
- **`trip_queue: VecDeque<Trip>`**, in FIFO order.
  - Each step, drop expired trips from the front (`tick − created_tick ≥ TRIP_EXPIRY_TICKS` → `unserved += 1`).
  - Then route up to `SPAWN_ROUTE_BUDGET` trips from the front. No route gives `unserved += 1`. Otherwise the routed trip moves to its origin link's queue.
- **`link_queues: BTreeMap<LinkId, VecDeque<RoutedTrip>>`**, where `RoutedTrip { trip, route: Vec<LinkId> }`.
  - Each step, in key order: drop expired trips at the front (counted as unserved), then try to spawn the front trip.
  - `Blocked` leaves it queued. Any other error drops it as unserved.
  - Empty queues are removed.

## Stats (`src/stats.rs`)

- **Per vehicle:** `free_flow: Vec<f64>`, set at spawn to `Σ free_time(link)` over the route. After a reroute, it becomes the time already elapsed plus `Σ free_time` over the new route.
- **`StatsWindow`:** `start_tick`, `arrivals`, `travel_sum`, `delay_sum`, `accrued_delay`, `spawned`, `unserved`, `stranded`, with `reset(tick)`.
  - An **arrival** is a despawn at the end of the last link. It adds `travel = (tick − spawn_tick) × DT` and `delay = max(0, travel − free_flow)`.
  - `accrued_delay` grows every step by `Σ DT × max(0, 1 − v/v0)` over live vehicles.
  - Stranded vehicles and unserved trips are only counted here. `UNSERVED_DELAY` is applied in M12 scoring.
- **`StatsSnapshot`** (ts-rs exported): `tick`, `sim_time_s`, `active`, `spawned`, `arrivals`, `unserved`, `stranded`, `mean_travel_s`, `mean_delay_s`, `accrued_delay_s`, `throughput_per_hour`, `mean_speed`, `stopped_share`.
  - Means over zero arrivals are 0.
  - `throughput_per_hour = arrivals / window_hours`.
  - `mean_speed` and `stopped_share` (`v < 0.5`) are instantaneous, over live vehicles.
  - `Sim::stats()` builds it. `Sim::reset_stats()` resets the window.
- **`Sim::road_speed_ratio(&self, out: &mut Vec<f32>)`:** resizes to `road_count`. Each road's value is the mean over link-placed vehicles on that road of `v / free_speed`, or 1.0 when the road is empty. Accumulate in one pass, in slot order.

## Step order after M7b

1. Rebuild the graph, landmarks (only at `new`) and demand tables if the network version changed.
2. Prefetch junctions and reroute invalid routes (M7a).
3. Arrivals, then trip creation.
4. Route up to the budget, then fill the link queues.
5. Spawn from the link queues.
6. Occupancy, zone bookkeeping, decisions, acceleration, integration and transitions (M5 and M6).
7. Stats accrual, EMA refresh and compaction, then `tick += 1`.

`state_hash` also covers:
- the arrival schedule (`next_arrival_s` bits)
- the queue lengths, and every queued trip's `(from, to, created_tick)`
- the window counters

## Tests

1. **`demand_weights`:** on a 3-road `MapBuilder` map with known lengths and classes, 100,000 samples match the weight ratios within 2 %.
2. **`poisson_rate`:** 3,600 vph over 36,000 ticks produces 3,600 arrivals ± 5 %. Rate 0 produces none.
3. **`trip_band`:** on `four_way(2, 200.0)` in city mode, every trip is unserved because no destination is ≥ 1 km away.
4. **`queue_expiry`:** a link queue blocked by a stopped vehicle, held at a red signal on `four_way`, drops trips after 600 ticks and counts them unserved.
5. **`region_mode`:** on `grid_city(20×20 @ 150)` with a region of radius 600 at the centre:
   - Only active links carry vehicles.
   - Sources are chosen at least 3× more often than their length share.
   - Vehicles that reach a sink despawn as arrivals.
6. **`stats_low_demand`:** `corridor()` with 60 vph, spawned via demand with the city band disabled through a test-only config `SimMode::City` plus `trip_band_override: Option<(f64, f64)>` behind `fixtures`, for 6,000 ticks.
   - `mean_delay_s < 10`.
   - `arrivals` equals despawns at the destination.
   - `stopped_share < 0.05`.
7. **`road_speed_ratio`:** an empty map gives all 1.0. A queue at a red light gives less than 0.2 on that road.
8. **`determinism_city`:** `grid_city(40×40 @ 150)` in city mode at 20,000 vph for 3,000 ticks, run twice, gives equal hashes. Also compare the `StatsSnapshot`.

## Example (`examples/bench.rs`, replacing ad-hoc timing)

- **Arguments:** `bench <map.bin.gz> [--region x,y,r] [--vph N] [--warmup N] [--ticks N]`.
- **Reports** (step timing with `Instant` outside `sim-core`):
  - median, p95 and max step ms
  - mean and max active vehicles
  - `spawned`, `arrivals`, `unserved` and `stranded`
  - routes per step and mean route µs
  - junctions built
- **Runs to report:**
  1. City: `120x120@150` synthetic, `--vph 100000 --warmup 6000 --ticks 3000`. The target is 20k active vehicles, median step ≤ 12 ms native, and route work ≤ 4 ms per step.
  2. Region: the same map, `--region 9000,9000,2000 --vph 12000 --warmup 3000 --ticks 3000`. The target is a median step ≤ 2 ms native.
- When a target is missed, report the numbers and the top cost from a quick profile (`perf` if available, otherwise timing splits added to the example). Don't change the spec's algorithms without the orchestrator.

## Acceptance

- `bash scripts/check.sh` passes.
- Report both bench outputs.
- No comments. Do not modify `CLAUDE.md`, `docs/` or `.claude/`.
