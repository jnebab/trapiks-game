# Performance

Measured after M17 on the real Metro Manila map (138,807 roads, 249,051 active links), in this development container: 4 cores, no GPU.

Browser numbers come from headless Chromium with software rendering (swiftshader), so treat them as relative. The synthetic numbers from M15c are kept at the end for comparison.

## Native sim (`examples/bench`)

**City run:** 85,000 vph, about 20.3k active vehicles, 9,000-tick warm-up, 1,000 measured ticks.

| Measure | Result | Target | Status |
|---|---|---|---|
| Step median | 10.33–10.55 ms | ≤ 12 ms | met |
| Step p95 | 15.2–15.4 ms | – | – |
| Step max | 22.6–25.8 ms | – | – |
| Mean route (band-limited city trips) | 0.98 ms (4,370 expansions) | ≤ 0.5 ms | missed |
| Route work per step | 2.2 ms (2.6 ms measured inside the step) | ≤ 4 ms | met |

**Time per phase** (mean ms per step), measured before M17's routing fix at 85k vph:

| Phase | ms |
|---|---|
| Route trips | 4.42 (2.63 after M17) |
| Accelerations | 2.05 |
| Decisions | 1.52 |
| Occupancy rebuild | 1.20 |
| Lane changes | 1.03 |
| Zone updates | 0.58 |
| Advance | 0.54 |
| Refresh ahead | 0.51 |
| Everything else | < 0.2 each |

**Routing diagnostics** (`examples/route_diag`): 2,000 seeded city trips of 1–12 km.

| Costs | Search | Before M17 | After M17 | Excess vs Dijkstra |
|---|---|---|---|---|
| Free-flow | weight 1, ALT | 1,024 µs / 4,534 exp | – | 0 |
| Free-flow | weight 2, ALT (game) | 1,250 µs / 7,312 exp | 665–710 µs | 16.5 % |
| Warmed | weight 1, ALT | 2,253 µs / 10,599 exp | – | 0 |
| Warmed | weight 2, ALT (game) | 1,677 µs / 9,608 exp | 889–944 µs / 4,170 exp | 10.4 % |

What the diagnostics show:
- The ALT bound divided by the true cost is 0.77 at free-flow and 0.64 in congestion.
- Before M17, weighted A* re-expanded closed links, so it did more work than unweighted A*. A closed set fixed that, and it was the main gain of M17.
- Planar landmark selection and a global congestion factor did not help and were reverted. The factor measured 0.985, because side streets dominate network length.

**Edits during the city run:**

| Edit | Max step | Target | Status |
|---|---|---|---|
| Delete the busiest primary road | 33.2 ms (was 56–59 ms) | ≤ 50 ms | met |
| Roundabout at a busy node | 317 ms | none; full-rebuild path | – |
| Add a road that splits a busy road | 270 ms | none; full-rebuild path | – |

What made the delete faster:
- The route graph now patches the affected links in place.
- Demand reachability now updates from a seed.
- That update now runs in the step after the edit.

**Region runs.** The challenges at 600–700 m run 2,000–5,000 vph. A baseline plus a 600 s evaluation takes 1.7–2.4 s of native wall time.

## Wasm sim (headless Chromium)

| Run | Vehicles | Step p50 | Step p95 | Max | Target | Status |
|---|---|---|---|---|---|---|
| City, run 1 | 20,711 | 18.0 ms | 29.1 ms | 60.3 ms | p50 ≤ 25 ms | met |
| City, run 2 | 21,005 | 17.7 ms | 29.7 ms | 84.3 ms | p50 ≤ 25 ms | met |

## Map and load

| Measure | Result | Target |
|---|---|---|
| Map | 138,807 roads, 5.46 MB gzipped | about 150k roads, ≤ 20 MB |
| Load to ready | 4.4–4.7 s alone, 6.6 s under test load | ≤ 6 s |
| Load to first tiles | 5.3 s alone, 7.9 s under test load | – |
| Junction build, whole map (native) | about 550 ms | – |

At city zoom, the whole map renders at 60 FPS in swiftshader once the tiles are built. The 60 fps street-zoom and 30 fps city-zoom render targets still need a real GPU to check.

## Open items

1. **Mean A* is about 1 ms against the 0.5 ms target.** Route work per step (2.2 ms) is well inside its 4 ms budget, so this does not block. Reaching 0.5 ms needs a different search design: bidirectional search, contraction or hub labels, or better landmarks than farthest-point or planar selection.
2. **Edits that append roads** (roundabouts, new roads) still take the full-rebuild path, at 270–320 ms on the real map. Patch-based rebuilds for them would work like the delete path.
3. **Challenge scoring:** 5 of 7 Metro Manila challenges have no 1-star fix. See the Outcome section of `docs/milestones/M16.md`.

## Synthetic 120×120 city (M15c, for comparison)

At 100k vph with about 23.8k vehicles:

| Measure | Result |
|---|---|
| Step median | 19.7 ms |
| Mean A* | 1.0 ms |
| Wasm p50 | 37.1 ms |
| Delete-road | 45.3 ms |

The synthetic grid is denser in vehicles per link than the real map, so per-vehicle passes dominated there. On the real map, routing dominated until M17.
