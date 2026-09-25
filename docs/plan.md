# Trapiks: master plan

## 1. Definition of done

A browser game titled **Trapiks**, written in Rust→WASM (simulation) and TypeScript (everything else, no UI framework), that:

1. Loads the real Metro Manila road network from OpenStreetMap: every drivable street, plus highways, expressways, skyways, ramps and intersections. It also loads water and parks.
2. Runs a microscopic traffic simulation over the whole network.
3. Lets the player fix traffic by deleting and modifying roads and intersections. Modifications include lanes, direction, speed, intersection control, signal timing, turn bans, flyovers and roundabouts. The player can also draw new connector roads.
4. Looks close to Trafficity (see §4).
5. Is playable end to end: title screen, challenges with a budget and a target, scoring, a sandbox, and saving.
6. Keeps the code clean: no comments, low cognitive complexity, and small modules, enforced by lint gates and review.

## 2. Roles and process

- **Orchestrator** (main session): writes the plans, runs checks, commits, and pushes to `claude/brave-pasteur-hlfvly` after each approved milestone.
- **Implementer** (`.claude/agents/implementer.md`, Opus 5.5, low effort): implements one milestone spec. It never commits.
- **Reviewer** (`.claude/agents/reviewer.md`, Fable 5.1, high effort): reviews every plan before implementation and every diff after it. Work loops until the verdict is `APPROVE`.
- Each milestone gets its own spec file, `docs/milestones/Mx.md`, which is reviewed before any code is written.

## 3. Architecture

### 3.1 Repository layout

```
Cargo.toml                    workspace
crates/
  sim-core/                   trapiks-sim-core: pure Rust, no wasm/web deps
  sim-wasm/                   trapiks-sim-wasm: wasm-bindgen facade
  mapgen/                     trapiks-mapgen: native CLI, OSM → map file
scripts/
  build-wasm.sh               cargo build → wasm-bindgen --target web → wasm-opt; exports ts-rs types
  fetch-osm.sh                Overpass query → data/osm/metro-manila.json (gitignored)
  build-map.sh                mapgen → web/public/maps/metro-manila.bin.gz (committed)
web/
  public/maps/                committed game map
  src/
    main.ts                   bootstrap only
    app/                      screens and game flow (title, challenge, sandbox)
    sim/                      worker client, protocol, snapshot interpolation
    worker/                   sim.worker.ts, fixed-timestep loop
    render/                   PixiJS: camera, tiles, roads, areas, vehicles, overlays, buildings
    edit/                     tools, picking, selection
    hud/                      plain DOM widgets + CSS
    generated/                ts-rs output (gitignored, produced by build-wasm.sh)
    wasm/pkg/                 wasm-bindgen output (gitignored)
docs/
  plan.md, milestones/
```

### 3.2 Runtime

- **Main thread:** PixiJS rendering, HUD, input, and picking. It never runs sim logic.
- **Worker:**
  - Owns the wasm module.
  - Runs a fixed-timestep loop with `dt = 0.1 s`. It does 1, 2, 4 or 8 steps per 100 ms wall tick, depending on the speed setting, or none when paused.
  - After each wall tick it posts one snapshot.
- **Boundary rules:** as in CLAUDE.md.
  - Call `step` once per tick.
  - Commands go into a queue applied at the start of the next step.
  - Read SoA views over wasm memory and recreate them if `memory.buffer` changes.
  - Copy into transferable buffers.
  - Reserve vehicle capacity at load (`MAX_VEHICLES = 60_000`).
- **Snapshot:**
  - Fields: `tick`, `simTime`, `count`, `ids: Uint32Array`, `x, y, heading: Float32Array`, and `style: Uint8Array` (color index and vehicle kind).
  - Double-buffered in the worker. The main thread returns the buffers after use so they can be reused.
- **Interpolation:** the renderer matches vehicles by `id` between the last two snapshots and lerps position and heading by wall-clock fraction. Vehicles that are new or gone snap into or out of view.
- **Road geometry:**
  - Sent once at load as transferable typed arrays built by wasm from the map file, so there is a single parser.
  - After each edit, a `networkDelta` message lists the changed road records.

### 3.3 Map pipeline (offline)

1. `fetch-osm.sh` sends an Overpass query over the area with `ISO3166-2=PH-00`:
   - Highways: `motorway|trunk|primary|secondary|tertiary|unclassified|residential|living_street|road` and their `*_link`s.
   - Nodes tagged `highway=traffic_signals|stop|give_way`.
   - Water: `natural=water` and `waterway=riverbank`, plus coastline in the bbox.
   - Parks and grass: `leisure=park|golf_course`, `landuse=grass|recreation_ground|cemetery|forest|meadow`.
   - Output uses `out body geom`.
2. `mapgen`:
   - Projects to local meters with an equirectangular projection around the bbox centre. It uses f64 and stores f32; y points down.
   - Splits ways at shared nodes.
   - Merges chains of degree-2 nodes when their attributes are equal.
   - Derives per-road attributes:
     - class
     - `lanes_forward` and `lanes_backward` (0 means one-way), with class defaults when untagged
     - speed limit (`maxspeed`, or a class default)
     - layer (`layer` / `bridge` / `tunnel`)
     - a name index
   - Marks a junction as controlled when a signal, stop or give-way node lies on an incident road within 25 m.
   - Simplifies areas with Douglas–Peucker at 1.5 m and assembles the coastline into a sea polygon clipped to the bbox.
3. **Output:** a versioned `MapData` (serde + postcard, gzip), written to `web/public/maps/metro-manila.bin.gz`.
   - It is committed so builds never need the network.
   - The ODbL attribution appears in the game and in `web/public/maps/LICENSE`.
4. **Fallback:** if Overpass stays blocked, mapgen gains an `.osm.pbf` reader (`osmpbf` crate) for a user-uploaded extract.

### 3.4 `sim-core` modules

| Module | Responsibility |
|---|---|
| `geom` | `Vec2`, polyline arc length, point-at-distance, offset, Bézier sampling, segment intersection |
| `rng` | PCG32 seeded RNG (no external RNG crate) |
| `map` | `MapData` (serde) and load/save with a version check |
| `network::road` | Roads (SoA): class, lanes per direction, speed, layer, geometry offsets, cumulative lengths |
| `network::link` | Directed links derived from roads (a road gives a forward and/or backward link) |
| `network::junction` | Built lazily per node and invalidated on edit: movements (link→link, lane ranges, Bézier path, length), conflict points between movements, priority ranks |
| `network::signal` | Signal plans per signal cluster (signalized nodes within 30 m, grouped by union-find). Phases come from approach axes. Links inside a cluster are never stopped |
| `network::spatial` | Uniform grid index of roads and nodes, used for region queries |
| `network::edit` | Applies edit commands to the network and reports changed roads and nodes |
| `vehicle::store` | Vehicle SoA: id, kind, color, position (link/lane or movement, `s`), speed, route cursor, spawn time, free-flow time |
| `vehicle::idm` | IDM acceleration (δ = 4 written as x²·x², no `powf`) |
| `vehicle::lanes` | Per-lane ordered occupancy, leader lookup across link→movement→link |
| `vehicle::junction_rules` | Entry decision: signal state, yielding by priority rank and time gap at conflict points, spillback (don't block the box), a wait timeout |
| `vehicle::lane_change` | Mandatory lane changes plus MOBIL (M8) |
| `demand` | Weighted origin and destination tables by road class and length. A Poisson spawn rate (vehicles per hour) with trip lengths of 1–12 km |
| `routing` | A* over links with travel-time costs refreshed every 60 s from EMA link speeds, plus reroute on invalidation |
| `stats` | City-wide and per-region metrics: active count, arrivals per hour, mean travel time, mean delay, mean speed, stopped share |
| `cost` | Cost of each edit, and the budget ledger |
| `command` | The `Command` enum (ts-rs), the queue, and `CommandResult` |
| `challenge` | Challenge definition (ts-rs), baseline and evaluation windows, and scoring |
| `sim` | The `Sim` facade: `load`, `enqueue`, `step`, snapshot fill, `state_hash` |

- **Determinism:**
  - One seeded PCG32 is threaded explicitly.
  - Iteration uses `Vec` and id order only.
  - Transcendental functions come from `libm`.
  - Lazy caches are pure functions of network state.
  - `state_hash` is FNV-1a over the SoA plus the network version.

### 3.5 Commands (player edits)

| Command | Effect |
|---|---|
| `DeleteRoad { road }` | Removes a road. Vehicles on it despawn, and routes through it reroute. |
| `SetLanes { road, forward, backward }` | Adds or removes lanes, makes a road one-way or two-way, or reverses it. |
| `SetSpeedLimit { road, kph }` | Changes the speed limit. |
| `SetJunctionControl { node, control }` | Sets the control to `Uncontrolled`, `Yield`, `AllWayStop` or `Signal`. |
| `SetSignalTiming { node, greens, offset }` | Sets per-phase green times and the offset. |
| `SetTurnAllowed { node, from, to, allowed }` | Bans or allows a turn, including U-turns. |
| `BuildFlyover { node, through: [road, road] }` | Splits the node so the through pair passes over; the pair renders elevated near the node. |
| `BuildRoundabout { node, radius }` | Replaces the node with a ring of one-way links (M9). |
| `AddRoad { from, to, control, lanes, layer }` | Adds a road; an endpoint can be a node or a point on a road, which splits that road (M9). |
| `Undo` | Reverts the last edit. Edits store their inverse. |
| `SetDemand { vehicles_per_hour }` | Changes the demand rate (sandbox only). |

- Every command is validated in `sim-core`, costed, and applied at the next step.
- The result is `Ok { cost, changed_roads, changed_nodes }` or `Err { reason }`.

### 3.6 Game loop

- **Challenges:** named hotspots such as EDSA–Ortigas, Magallanes, Balintawak, EDSA–Quezon Ave, C-5–Kalayaan, España–Lacson and Taft–Buendia. Each has:
  - a centre (lat/lon) and a radius
  - a budget
  - a target, e.g. −25 % mean delay in the region
  - a demand rate
- **Flow:**
  1. Pick a challenge.
  2. The camera flies to it.
  3. A baseline runs: 10 sim-minutes of warm-up, then 10 minutes of measurement, at maximum speed.
  4. The player edits.
  5. The player presses Evaluate, which runs the same warm-up and measurement.
  6. The result screen shows the percentage improvement, cost, and 1–3 stars.
- **Sandbox:** the whole city, a large budget, a demand slider, and city-wide stats.
- **Saves:** `localStorage` keeps the seed and the command log for each challenge. Replay is deterministic.
- **Traffic layer:** at city zoom, roads are tinted by the speed ratio from worker stats so the player can find jams.

### 3.7 Performance targets and scale

- **Map size:** about 100k roads and 500k geometry points.
  - The gzip-compressed map should be at most 15 MB.
  - Load and parse should take at most 5 s.
- **Sim:** 20k active vehicles.
  - Native `step` should take at most 12 ms, and the wasm `step` at most 25 ms.
  - A* per trip should average at most 3 ms.
  - Measure with `cargo run --release --example bench`.
- **Render:**
  - 60 fps at street zoom and at least 30 fps at city zoom.
  - Static geometry is built per 512 m tile in 3 LOD bands:
    - **city:** major roads as single strokes
    - **district:** all roads with outlines
    - **street:** markings, yield lines, signals, node dots and buildings
  - Tiles are built lazily and kept in an LRU cache of about 256 tiles.
  - Vehicles use a `ParticleContainer`.

## 4. Visual style spec (from the Trafficity reference)

**Reference screenshots** (local only, never committed): `/tmp/claude-0/-home-user-trapiks-game/74a64fa3-bf2f-5a5c-88ae-f83b4d67c7ec/scratchpad/style-ref/{1,2,3}.webp`

**Ground and areas:**

| Element | Colour |
|---|---|
| Ground | `#EEEDE6` |
| Parks and grass | `#CDE6A6` |
| Water | `#A8C0F4` (no outline) |
| Sand | `#F2E6AC` |

**Roads:**
- **Surface:** white `#FFFFFF`. Width is lanes × 3.2 m.
- **Ground-level edge:** 0.35 m grey `#9B9B97`.
- **Elevated roads** (layer > 0):
  - a 0.5 m black edge `#151515`
  - a soft shadow `rgba(0,0,0,0.13)` offset (+3 m, +3 m) per layer
  - drawn above ground roads, in layer order
- **Junctions:**
  - Every road in a pass draws all outlines first, then all fills, so junctions merge seamlessly.
  - Round joins and butt caps.

**Markings:**

| Marking | Style |
|---|---|
| Two-way centre line | Yellow `#E2C23D`, 0.18 m |
| Lane dividers | Dashed grey `#8E8E8E`, 0.15 m, 3 m dash / 3 m gap |
| Yield/stop lines | Dashed dark `#3A3A3A` across the approach at junction entry |

**Signals:** small rounded pills at the stop line, in green `#43B581`, amber `#F2B33D` or red `#E5484D`.

**Node handles (edit mode):** black dots `#151515`, radius 0.6 m, at road endpoints.

**Vehicles:**
- **Car:** a 4.4 × 1.9 m rounded rect with a dark windshield bar `#1F2330` at 70 % of its length.
- **Palette:** `#E8616F #F28C38 #F2C94C #6FCF97 #56CCF2 #5B8DEF #BB6BD9 #EB7FB5 #4FB3A9`
- **Other kinds:** jeepney 6.5 × 2.1 m (white body, colour stripe) and bus 12 × 2.5 m `#4A7BD0` (M8).

**Buildings** (M9, procedural along residential roads at street zoom):
- **Houses:** light green `#8FD16A` with darker green `#62A83E` gable roof faces.
- **Commercial:** blue `#4F7FD8` along primary and secondary roads.
- **Warehouses:** dark grey `#5A5A5A`.
- **Lots:** pale `#F4F4EE` with white parking stripes.
- **Shadows:** offset down-right, `rgba(0,0,0,0.13)`.

**UI:**
- **Chips:** white, 1 px `#D6D6D0` border, radius 6 px.
- **Text:** Lexend 500 (bundled via `@fontsource/lexend`) in `#1D1D1D`, with a subtle shadow.
- **Tooltip chips** look like `$60` and `Add lane`, and appear next to the cursor.

## 5. Quality gates (enforced in CI and by the reviewer)

**Rust:**
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings` with `clippy.toml`: `cognitive-complexity-threshold = 10`, `too-many-lines-threshold = 40`, `too-many-arguments-threshold = 5`
- `#![deny(clippy::unwrap_used, clippy::expect_used)]` in library code, outside tests

**TypeScript:**
- `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`
- ESLint flat config:
  - `typescript-eslint` strictTypeChecked
  - `sonarjs/cognitive-complexity: 8`
  - `complexity: 8`
  - `max-depth: 2`
  - `max-lines-per-function: 40`
  - `max-lines: 200`
  - `max-params: 4`
- Prettier

**Comments:** `scripts/check-no-comments.mjs` uses the TypeScript scanner for TS and a Rust token scan for `//` and `/*`. It allows only `// SAFETY:`. Doc comments are also disallowed.

**Tests:**
- `cargo test` for everything
- `vitest` for pure TS modules
- a Playwright smoke test (Chromium at `/opt/pw-browsers`) from M4 onward

**CI:** `.github/workflows/ci.yml` runs every gate plus `build-wasm.sh` and `pnpm build`.

## 6. Milestones

Each milestone is small enough to review. Each ends with green gates, an approving review, a commit, and a push.

| # | Milestone | Acceptance |
|---|---|---|
| M0 | Tooling and scaffold: workspace, empty crates, Vite+TS, lint/format/complexity/no-comment gates, pinned wasm toolchain, `build-wasm.sh`, CI, updated CLAUDE.md | All gates pass on the skeleton. `pnpm --dir web build` produces a page that loads the wasm and shows "Trapiks" |
| M1 | Map pipeline: `MapData` in sim-core, mapgen (Overpass JSON → MapData), `fetch-osm.sh`, `build-map.sh`, fixtures, committed map | Fixture tests: splitting, merging, one-way, lanes defaults, layers, signal assignment, coastline assembly. The real map is generated, and its stats (roads, nodes, points, size) are printed and within targets |
| M2 | Network runtime: roads, links, lazy junctions (movements, lane ranges, Bézier paths, conflicts, priorities), signal clusters, spatial index | Unit tests on fixture junctions (Phase-0 4-way with 2 lanes each way, T-junction, dual-carriageway cluster, one-way pair). Every junction in the real map builds without panics, with timing reported |
| M3 | Vehicles and traffic: store, IDM, lane occupancy, junction rules, spillback, demand, A* routing, arrivals, stats, `state_hash` | Determinism test (same seed + commands ⇒ same hash after 5,000 ticks, twice, on the fixture and the real map). Invariants: no NaN, no overlap, no vehicle stuck for more than 300 s on the fixture at low demand. The benchmark meets §3.7 |
| M4 | WASM + worker pipeline: wasm API, ts-rs types, worker loop with speed control, snapshot double-buffering, interpolation, a debug renderer (rects), FPS and vehicle-count overlay | The browser shows the real map with moving vehicles. A Playwright smoke test checks that the vehicle count is above 0 after 5 s and the FPS is above 30 |
| M5 | Trafficity-style renderer: camera (pan, zoom, inertia, fly-to), tiles + LOD, roads/markings/elevation/shadows, areas, vehicle particles, signal pills | A Playwright screenshot at street zoom on EDSA–Ortigas is compared by eye against the reference. Frame budget met at city zoom |
| M6 | Editing: all §3.5 commands except roundabout/add-road, costs + budget, undo; picking, selection highlight, tool palette, inspector, tooltip chips, network deltas → tile rebuild | Unit tests per command, including the inverse. Playwright: delete a road and the vehicles reroute; add a lane and the tile updates |
| M7 | Game loop: title screen, challenge list, baseline/evaluate/score, sandbox, save/load, traffic layer, attribution | Playwright plays one challenge end to end. Replay from save reproduces the same hash |
| M8 | Lane changes (MOBIL + mandatory) and vehicle kinds (car, jeepney, bus) | Tests: lane-change safety (no overlap), and mandatory lane changes reach the turn lane. Determinism still holds |
| M9 | Polish: procedural buildings, roundabout, add-road (straight + curved, elevated), performance pass, GitHub Pages deploy workflow | Playwright covers the roundabout and add-road flows. Final performance numbers are recorded in `docs/perf.md` |

## 7. Risks

| Risk | Mitigation |
|---|---|
| Overpass is blocked by network policy | The user allows `overpass-api.de`. Until then, M1 is built and tested on fixtures, and M2 and M3 proceed. Fallback: a PBF reader for an uploaded extract |
| Gridlock or deadlock in dense grids | Spillback rule, a wait timeout that relaxes yielding, and a demand cap in challenges |
| Signal placement in OSM sits near junctions, not on them | Assign signals to junctions within 25 m and cluster signalized nodes within 30 m |
| Render cost of about 100k roads | Tiles, LOD bands, an LRU cache, and a single major-road mesh at city zoom |
| Routing cost | A* with a Euclidean/vmax heuristic and bounded trip lengths. ALT landmarks only if the benchmark fails |
| wasm-bindgen CLI/crate mismatch | Pin both to the same version; `build-wasm.sh` checks this |
| ODbL share-alike on the derived map | Attribution in the game and a LICENSE next to the map |
