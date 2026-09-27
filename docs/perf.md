# Performance

Measured after M15c on the synthetic `120x120@150` city (58,088 links), in this development container: 4 cores, no GPU. Browser numbers come from headless Chromium with software rendering (swiftshader), so they are only useful relative to each other.

The real Metro Manila map has not been generated yet (overpass-api.de is unreachable here). Re-measure on it once it exists.

## Native sim (`examples/bench`)

**City run:** 100k vph, about 23.8k active vehicles, 9,000-tick warm-up.

| Measure | Result | Target | Status |
|---|---|---|---|
| Step median | 19.7 ms (repeat run 21.9 ms) | ≤ 12 ms | missed |
| Step p95 | 26.7 ms | – | – |
| Mean A* route | 1.0 ms | ≤ 0.5 ms | missed |

Time per phase (mean ms per step):

| Phase | ms |
|---|---|
| Accelerations | 6.25 |
| Decisions | 3.74 |
| Lane changes | 3.10 |
| Routing | 1.65 |
| Occupancy rebuild | 1.63 |

**Region run:** 2 km, 36k vph, about 3.5k active vehicles.

| Measure | Result | Target | Status |
|---|---|---|---|
| Step median | 1.23 ms | ≤ 2 ms | met |
| Step p95 | 1.87 ms | – | – |
| Mean A* route | 86 µs | – | – |

**Edits during the city run:**

| Edit | Max step | Target | Status |
|---|---|---|---|
| Delete the busiest primary road | 45.3 ms (was 130 ms) | ≤ 50 ms | met |
| Radius-18 roundabout at the busiest node | 173 ms | none; appends roads, so it takes the full-rebuild path | – |
| Add a road that splits the busiest road | 121 ms | none; same reason | – |

## Wasm sim (headless Chromium)

| Run | Step p50 | Step p95 | Target | Status |
|---|---|---|---|---|
| City, 23.3k vehicles | 37.1 ms | 60.3 ms | ≤ 25 ms | missed |
| Region, 4.8k vehicles | 3.8 ms | 7.1 ms | ≤ 2 ms at 3k vehicles | missed, but measured at 4.8k |

## Render and load (swiftshader, relative only)

| Measure | Result |
|---|---|
| City zoom, whole map: frame p50 / p95 | 16.7 / 100 ms |
| Street zoom, 23k vehicles: frame p50 / p95 | 100 / 100 ms |
| Load to ready, `synthetic` | 452 ms |
| Load to first tiles, `synthetic` | 571 ms |
| Load to ready, `grid120` | 1,821 ms |
| Load to first tiles, `grid120` | 2,529 ms |

The 60 fps and 30 fps render targets can only be checked on a real GPU.

## Next steps

These were proposed but not implemented:
1. Cache each vehicle's leader and gap once per step, and share it between the acceleration, decision and lane-change passes.
2. Evaluate lane changes only for vehicles within a window of their next decision zone.
3. Improve the A* heuristic with more landmarks or congestion-aware bounds, or cache routes per origin and destination.
4. Add patch-based rebuilds for edits that append roads (roundabouts, new roads), like the delete path now has.
