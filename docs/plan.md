# Trapiks: master plan

## 1. Definition of done

A browser game titled **Trapiks**, written in Rust→WASM (simulation) and TypeScript (everything else, no UI framework), that:

1. Loads the real Metro Manila road network from OpenStreetMap: every drivable street, plus highways, expressways, skyways, ramps and intersections. It also loads water and parks.
2. Runs a microscopic traffic simulation: the whole city in the sandbox, or a region subgraph in challenges.
3. Lets the player fix traffic by deleting and modifying roads and intersections. Modifications include lanes, direction, speed, elevation, intersection control, signal timing, turn bans, flyovers and roundabouts. The player can also draw new connector roads.
4. Looks close to Trafficity (§4).
5. Is playable end to end: title screen, challenges with a budget and a target, scoring, a sandbox, and saving.
6. Keeps the code clean: no comments, low cognitive complexity, and small modules, enforced by lint gates and review.

## 2. Roles and process

- **Orchestrator** (main session): writes the plans, runs checks, commits, and pushes to `claude/brave-pasteur-hlfvly` after each approved milestone.
- **Implementer** (`.claude/agents/implementer.md`, Opus 5.5, low effort): implements one milestone spec. It never commits.
- **Reviewer** (`.claude/agents/reviewer.md`, Fable 5.1, high effort): reviews every plan before implementation and every diff after it. Work loops until the verdict is `APPROVE`.
- Each milestone gets its own spec file, `docs/milestones/Mx.md`, which is reviewed before any code is written. Specs copy the fixed decisions in §3 verbatim instead of re-deciding them.
- `scripts/check.sh` runs every gate. CI, the implementer and the reviewer all run this same script.

## 3. Architecture

### 3.1 Repository layout

```
Cargo.toml                    workspace
crates/
  sim-core/                   trapiks-sim-core: pure Rust, no wasm/web deps
  sim-wasm/                   trapiks-sim-wasm: wasm-bindgen facade
  mapgen/                     trapiks-mapgen: native CLI, OSM → map file
scripts/
  check.sh                    every quality gate
  build-wasm.sh               cargo build → wasm-bindgen --target web → wasm-opt; exports ts-rs types
  check-no-comments.mjs       comment gate for TS and Rust
  fetch-osm.sh                quadrant-split Overpass queries → data/osm/*.json (gitignored)
  build-map.sh                mapgen → web/public/maps/metro-manila.bin.gz (committed)
web/
  public/maps/                committed game map + LICENSE (ODbL)
  src/
    main.ts                   bootstrap only
    app/                      screens and game flow (title, challenge, sandbox)
    sim/                      worker client, protocol, snapshot interpolation
    worker/                   sim.worker.ts, fixed-timestep loop, map fetch + gunzip
    render/                   PixiJS: camera, tiles, roads, areas, vehicles, overlays, buildings
    edit/                     tools, picking grid (main thread), selection
    hud/                      plain DOM widgets + CSS
    generated/                ts-rs output (gitignored, produced by build-wasm.sh)
    wasm/pkg/                 wasm-bindgen output (gitignored)
docs/
  plan.md, milestones/, perf.md
```

### 3.2 Runtime and the wasm boundary

**Main thread:** PixiJS rendering, HUD, input, and picking. Picking uses a TypeScript uniform grid over the received road arrays, so hovering never waits on the worker. The main thread never runs sim logic.

**Worker:**
- Fetches `metro-manila.bin.gz` and gunzips it with `DecompressionStream`, so the bytes go straight into `Sim::load`.
- Runs a fixed-timestep loop with `dt = 0.1 s`. A wall tick is 100 ms, and the speed setting decides how many steps run in it:

  | Speed | Steps per wall tick |
  |---|---|
  | paused | 0 |
  | 1× | 1 |
  | 2× | 2 |
  | 4× | 4 |
  | 8× | 8 |
  | max | as many as fit in 90 ms of wall time |

- Speeds are best-effort: the worker never runs more steps than fit in 90 ms, so there is no spiral of death.

**Boundary rules** (as in CLAUDE.md):
- Call `step` once per sim tick, never per vehicle.
- Commands go into a queue applied at the start of the next step.
- Read SoA views over wasm memory and recreate them whenever `memory.buffer` changes.
- Copy into transferable buffers.
- Reserve vehicle capacity at load (`MAX_VEHICLES = 60_000`).

**Messages from the worker:**

| Message | When | Contents |
|---|---|---|
| `ready` | once | Road render arrays (transferable), node arrays, names, areas, map meta |
| `snapshot` | every wall tick | `tick`, `simTime`, `count`, `ids: Uint32Array`, `x, y, heading: Float32Array`, `style: Uint8Array` (kind and colour index) |
| `networkDelta` | after each applied edit | Changed road and node records |
| `stats` | 1 Hz | City or region metrics, plus `roadSpeedRatio: Float32Array(roadCount)` for the traffic layer |
| `commandResult` | per command | `Ok { cost, changed_roads, changed_nodes }` or `Err { reason }` |

**Buffers:** snapshots are double-buffered, and the main thread returns the buffers so they can be reused.

**Interpolation:**
- The renderer matches vehicles by `id` between the last two snapshots.
- Position is lerped by wall-clock fraction. Heading is lerped along the shortest arc.
- New vehicles appear at their position. Gone vehicles disappear.

**Memory:** the wasm heap stays at or below 512 MB with 20k vehicles on the full map.

### 3.3 Map pipeline (offline)

**1. Fetch.** `fetch-osm.sh` sends Overpass queries over the bbox of `ISO3166-2=PH-00`. The bbox is split into 4 quadrants, and each query uses `[timeout:900][maxsize:1073741824]` and `out body geom`. The queries fetch:
- **Roads:** ways with `highway=motorway|trunk|primary|secondary|tertiary|unclassified|residential|living_street|road|service` and every `*_link`. Excluded:
  - `service=parking_aisle|driveway|drive-through|emergency_access`
  - `access=private|no`
  - `area=yes`
- **Control nodes:** nodes tagged `highway=traffic_signals|stop|give_way`.
- **Turn restrictions:** `relation[type=restriction]` (`no_*` and `only_*`).
- **Water:** ways and relations with `natural=water` or `waterway=riverbank`, plus `natural=coastline` ways.
- **Parks and grass:** ways and multipolygon relations with `leisure=park|golf_course` or `landuse=grass|recreation_ground|cemetery|forest|meadow`.

The fallback is a user-uploaded `.osm.pbf` read with the `osmpbf` crate.

**2. Build.** `mapgen` converts the OSM data into the game map:

- **Projection:** equirectangular around the bbox centre. Computed in f64 and stored as f32 meters, with y pointing down.
- **Road splitting:** ways are split at shared nodes. Chains of degree-2 nodes are merged when their attributes are equal.
- **Road attributes:**

  | Attribute | Derivation |
  |---|---|
  | Class | From the `highway` tag |
  | Lanes | `lanes_forward` and `lanes_backward`, from `lanes`, `lanes:forward` and `lanes:backward`; class defaults when untagged |
  | One-way | `oneway=yes`; `oneway=-1` reverses the road; `junction=roundabout` and `motorway` imply one-way |
  | Speed | `maxspeed`, or a class default |
  | Layer | `layer`; defaults to `bridge ⇒ 1` and `tunnel ⇒ -1` |
  | Name | Index into a names table |

- **Junction control:** a junction is signalized, stop-controlled or yield-controlled when a matching control node lies on an incident road within 25 m.
- **Turn restrictions:** converted to banned (from road, via node, to road) triples. `only_*` expands into bans on every other turn.
- **Areas:**
  - Outer rings of multipolygons are assembled from member ways. Inner rings are skipped until M15.
  - Coastline ways are joined and closed into a sea polygon along the bbox. OSM coastlines have land on the left of the way direction and water on the right. The y-down projection flips the sign of the 2D cross product, so compute the side in lon/lat, or negate it.
  - Rings are simplified with Douglas–Peucker at 1.5 m.

**3. Output.** A versioned `MapData` (serde + postcard, gzip), written to `web/public/maps/metro-manila.bin.gz`. It includes a `map_hash`, the FNV-1a hash of the uncompressed bytes.
- The file is committed, and regenerated rarely, so builds never need the network.
- The ODbL attribution appears in the game and in `web/public/maps/LICENSE`.

### 3.4 `sim-core` modules

| Module | Responsibility |
|---|---|
| `geom` | `Vec2`, polyline arc length, point-at-distance, offset, Bézier sampling, segment intersection |
| `rng` | PCG32 seeded RNG (no external RNG crate) |
| `map` | `MapData` (serde), load/save, version and hash check |
| `network::road` | Roads (SoA): class, lanes per direction, speed, layer, geometry offsets, cumulative lengths, `deleted` flag |
| `network::link` | Directed links derived from live roads |
| `network::junction` | Built lazily per node and invalidated on edit: movements, lane ranges, Bézier paths, conflict points, priority ranks |
| `network::signal` | Signal clusters and phase plans |
| `network::region` | Active-road mask for a challenge circle, plus boundary source and sink links |
| `network::spatial` | Uniform grid of roads and nodes |
| `network::edit` | Applies edit commands. Each edit returns its inverse and the changed ids |
| `vehicle::store` | Vehicle SoA: id, kind, colour, location (link+lane or movement), `s`, `v`, route `(offset, len)` into a flat route arena, cursor, spawn tick, free-flow time |
| `vehicle::idm` | IDM acceleration |
| `vehicle::lanes` | Per-lane ordered occupancy, and leader lookup across link → movement → link |
| `vehicle::junction_rules` | Entry decision (§3.5) |
| `vehicle::lane_change` | Mandatory lane changes plus MOBIL (M14) |
| `demand` | Origin and destination tables, spawn queues |
| `routing` | A*, EMA link costs, reroute budget |
| `stats` | City and region metrics |
| `cost` | Cost of each edit, and the budget ledger |
| `command` | The `Command` enum (ts-rs), the queue, and `CommandResult` |
| `challenge` | Challenge definition (ts-rs), run phases, and scoring |
| `sim` | The `Sim` facade: `load(map, mode)`, `reset`, `enqueue`, `step`, `fill_snapshot`, `state_hash` |

### 3.5 Fixed sim decisions

Milestone specs copy these decisions as written.

#### Units and time

- SI units: m, s, m/s.
- `dt = 0.1 s`. 600 ticks = 60 s.
- Vehicle state is f64. Render output is f32.
- Philippine traffic drives on the right.

#### IDM

| Parameter | Value |
|---|---|
| `v0` | Link speed limit |
| `T` | 1.2 s |
| `s0` | 2 m |
| `a` | 1.2 m/s² |
| `b` | 2.0 m/s² |
| `δ` | 4 (computed as `x² · x²`, never `powf`) |

- Integration is ballistic, with `v` clamped to ≥ 0.
- A car is 4.4 m long. Lanes are 3.2 m wide.
- Vehicle position = centreline point at `s` + perpendicular offset for the lane. Every lane uses the centreline arc length.

#### Lane ranges

Lanes on a link are indexed from the right, starting at 0.

| Lane count | Rule |
|---|---|
| 1 lane | That lane serves every movement |
| Several lanes | Rightmost lane: right turns and through. Leftmost lane: left turns, U-turns and through. Middle lanes: through only |
| No through movement | Split the lanes between the turn movements, right turns taking the right lanes |

- Before M14, a vehicle entering a link takes the lowest lane index within the lane range of its next movement.

#### Movements

- A movement is a path from an incoming link to an outgoing link: a cubic Bézier from the end of the incoming lane to the start of the target lane.
- U-turns are banned by default, except at dead ends (node degree 1). OSM turn restrictions are loaded as default bans.

#### Conflicts

- Each movement's Bézier is sampled into 8 segments. Two movements conflict where their samples intersect, or where they share a target link (a merge).
- A conflict point stores the distance along each movement.

#### Priority rank (higher wins)

1. Signal green.
2. Class rank of the incoming road.
3. Movement type: through, then right, then left, then U-turn.
4. Ties go to the earlier arrival tick, then the lower vehicle id.

#### Entry rule

- A vehicle decides at the stop line once it is within `v²/(2b) + 5 m` of it.
- It may enter when all of these hold:
  1. The signal allows it.
  2. No conflicting movement is occupied before its conflict point, or within 5 m past it.
  3. No higher-rank vehicle will reach a shared conflict point within 2.5 s.
  4. There is space: the last vehicle on the target lane has `s ≥ length + s0`.
- **Signal clusters:** a vehicle may enter a cluster only if there is space on the first link outside the cluster along its route.
- **Stop control:** a stop-controlled approach must first reach `v < 0.5`.
- **Yield control:** at a `Yield` node, approaches below the node's highest incoming class rank use a 4.0 s gap in rule 3 instead of 2.5 s. `Priority` nodes use 2.5 s for every approach.
- **Timeout:** after 300 ticks of waiting, rule 3 is waived, but rules 1, 2 and 4 still apply.
- A vehicle that has entered is committed.

#### Signals

- **Clusters:** signalized nodes within 30 m are grouped by union-find. Links inside a cluster are never stopped.
- **Phases:**
  - Built from the external approaches of the cluster.
  - Approaches are sorted into two heading bins (±35° around the first approach's axis), giving 2 phases.
  - If some approach fits neither bin, there is one phase per approach, in order of approach link id.
- **Timing:** green 30 s, amber 3 s, all-red 2 s. Offset 0.
- **Amber:** a vehicle stops if it can (distance > `v²/(2b)`); otherwise it continues.

#### Routing

- A* over links. Costs:
  - `length / ema_speed`
  - a left-turn penalty of 5 s and a U-turn penalty of 10 s
- The heuristic is Euclidean distance ÷ 27.8 m/s. Ties go to the lower link id.
- **EMA speeds:**
  - Refreshed every 600 ticks with α = 0.3 from the mean speed on each link.
  - A link with no vehicles drifts back toward free-flow speed.

#### Reroute budget per step

Budgets are counts, never time, so they stay deterministic.
1. Vehicles whose valid route prefix ends at their next junction reroute immediately, with no limit.
2. Vehicles flagged by an edit reroute next, at most 8 per step, in id order.
3. New spawns get at most 8 route computations per step.
4. Spawns that don't fit wait in the queue.

Invariant: route computations per step × the average A* time stays within 4 ms. A* must average ≤ 0.5 ms per trip. If the M7 bench misses that, M7 adds ALT: 8 landmarks chosen by farthest-point order from node 0, with ties broken by id. The landmark bounds are computed on free-flow costs, so they stay admissible because EMA costs never drop below free-flow.

#### Demand

**Origin and destination tables** are weighted by road length × class weight:

| Class | Weight |
|---|---|
| motorway | 8 |
| trunk | 6 |
| primary | 4 |
| secondary | 3 |
| tertiary | 2 |
| any other | 1 |

- In region mode, boundary source and sink links get 5× weight.

**Trips:**
- Straight-line trip length is 1–12 km in the city and 0.3–4 km in a region.
- A destination is drawn at most 8 times to land inside that band. If every draw misses, the spawn is dropped as unserved.
- Spawns follow a Poisson process at the mode's vehicles-per-hour rate.
- A vehicle enters at `s = 0` of its origin link when the gap allows. Otherwise it waits in a per-link queue, and after 600 ticks in the queue it is dropped as unserved.

#### Stats

- **Delay** for a trip = travel time − free-flow time along the route.
- **Region delay** is accrued per tick for each vehicle inside the region: `dt · max(0, 1 − v / v0)`.
- A vehicle that is dropped unserved adds 300 s of delay.
- **Throughput** = arrivals, or region exits, per hour.

#### Determinism

- One PCG32 is threaded explicitly.
- Iteration follows `Vec` order and ids only. Any order derived from floats is sorted by id with `sort_unstable_by_key`.
- Transcendental functions come from `libm`.
- Lazy caches are pure functions of network state.
- `state_hash` is FNV-1a over the vehicle SoA, the network version and the RNG state.

#### Id stability

- Roads and nodes are never removed from the SoA. `DeleteRoad` sets `deleted` and drops the road's links; undo clears the flag.
- `AddRoad`, `BuildFlyover` and `BuildRoundabout` append new roads and nodes. Only the undo of that same command truncates them.
- Saves store `map_hash` and reject a command log made against a different map.

### 3.6 Commands (player edits)

| Command | Effect |
|---|---|
| `DeleteRoad { road }` | Deletes a road. Vehicles on it despawn; vehicles routed through it are flagged for reroute |
| `SetLanes { road, forward, backward }` | Adds or removes lanes, makes a road one-way or two-way, or reverses it |
| `SetSpeedLimit { road, kph }` | Changes the speed limit |
| `SetLayer { road, layer }` | Changes elevation for render and crossing; connectivity is unchanged |
| `SetJunctionControl { node, control }` | Sets the control to `Priority`, `Yield`, `AllWayStop` or `Signal` |
| `SetSignalTiming { node, greens, offset }` | Sets green time per phase and the offset |
| `SetTurnAllowed { node, from, to, allowed }` | Bans or allows a turn |
| `BuildFlyover { node, through: [road, road] }` | Splits each through road 80 m from the node, appending the new ids. The two inner pieces reconnect at a new node at layer + 1 and bypass the junction |
| `BuildRoundabout { node, radius }` | Replaces the node with a ring of one-way links (M15) |
| `AddRoad { from, to, control, lanes, layer }` | Adds a road; an endpoint can be a node or a point on a road, which splits that road (M15) |
| `Undo` | Applies the stored inverse of the last edit |
| `SetDemand { vehicles_per_hour }` | Changes the demand rate (sandbox only) |

- Every command is validated in `sim-core`, costed, and applied at the start of the next step.
- In challenge mode, commands on roads outside the region are rejected.

### 3.7 Game loop

**Modes:**
- **Sandbox:** the whole city, with a large budget and a demand slider.
- **Challenge:** `Sim::load(map, Region { centre, radius })` activates only the roads within the radius. Road and node ids stay the same as in the full map.

**Challenges** are named hotspots: EDSA–Ortigas, Magallanes, Balintawak, EDSA–Quezon Ave, C-5–Kalayaan, España–Lacson and Taft–Buendia. Each defines:
- a centre (lat/lon) and a radius of 1.2–2.5 km
- a seed, used by both the baseline and the evaluation
- a budget
- a demand rate
- a target, such as −25 % mean region delay, with throughput not below 95 % of the baseline

**Challenge flow:**
1. Pick a challenge. The camera flies to it.
2. **Baseline:** reset, then 3,000 ticks of warm-up and 6,000 ticks of measurement, at max speed, behind a progress bar. At about 45 steps per wall tick, this takes about 20 s.
3. The player edits. Edits apply live so the player sees their effect.
4. **Evaluate:** reset with the same seed, apply the full command log at tick 0, then run the same warm-up and measurement.
5. The result screen shows the improvement, the cost and 1–3 stars.

**Saves:** `localStorage` keeps `map_hash`, the mode, the seed and the command log. Loading replays the log at tick 0, in every mode.

**Traffic layer:** at city zoom, roads are tinted by `roadSpeedRatio`.

### 3.8 Rendering

**LOD bands:**

| Band | Tiling | Contents |
|---|---|---|
| city | 4096 m tiles | Primary-class roads and above, as single strokes, tinted by the traffic layer |
| district | 512 m tiles | All roads with outlines |
| street | 512 m tiles | Everything in district, plus markings, one-way arrows, yield lines, signal pills, node dots and (M15) buildings |

**Tile ownership:** each road belongs to exactly one tile per band, the one containing the centre of its bbox. A tile's cull bounds are the union of its roads' bboxes, so roads are never drawn twice.

**Layering:** global containers are ordered per layer: `areas → [for each layer, ascending: shadow, outline, fill, markings] → selection → vehicles → overlays`. Every tile adds its Graphics to these shared containers, so all outlines sit under all fills and tile edges show no seams.

**Shadows:** each elevated layer's shadow container draws opaque shapes offset (+6, +8) m per layer, with `[BlurFilter, AlphaFilter(0.13)]` on the container. Overlapping shadows therefore don't darken twice.

**Tile builds:** at most 2 tiles are built per frame, and an LRU cache keeps about 256 tiles per band.

**Vehicles:** a single `ParticleContainer`. One atlas texture holds the car, jeepney and bus sprites, each with its windshield bar, and tint sets the body colour.

### 3.9 Performance targets

| Area | Target |
|---|---|
| Map | About 150k roads. At most 20 MB gzipped. Load and parse in at most 6 s |
| Region sim | Wasm step ≤ 2 ms with 3k vehicles |
| City sim | Native step ≤ 12 ms and wasm step ≤ 25 ms with 20k vehicles. A* averages ≤ 0.5 ms per trip |
| Edits | Deleting a primary road with 20k active vehicles never produces a native step over 50 ms |
| Render (manual numbers, recorded in `docs/perf.md`) | 60 fps at street zoom; ≥ 30 fps at city zoom |

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
  - a soft shadow, as in §3.8
  - drawn above lower layers
- **Tunnels** (layer < 0): drawn at 40 % alpha under the ground layer.
- **Joins and caps:** round joins; butt caps.
- **Dual carriageways:** OSM maps these as two one-way roads, so they render without a yellow centre line. This is accepted.

**Markings:**

| Marking | Style |
|---|---|
| Two-way centre line | Yellow `#E2C23D`, 0.18 m |
| Lane dividers | Dashed grey `#8E8E8E`, 0.15 m, 3 m dash / 3 m gap |
| Yield/stop lines | Dashed dark `#3A3A3A` across the approach at the junction entry |
| One-way arrows | Grey `#8E8E8E`, every 40 m |

**Signals:** small rounded pills at the stop line, in green `#43B581`, amber `#F2B33D` or red `#E5484D`.

**Edit overlays:**
- **Node handles:** black dots `#151515`, radius 0.6 m, at road endpoints.
- **Hover and selection highlight:** light cyan `#8ED8F6` over the road surface.

**Vehicles:**
- **Car:** a 4.4 × 1.9 m rounded rect with a dark windshield bar `#1F2330` at 70 % of its length.
- **Palette:** `#E8616F #F28C38 #F2C94C #6FCF97 #56CCF2 #5B8DEF #BB6BD9 #EB7FB5 #4FB3A9`
- **Other kinds:** jeepney 6.5 × 2.1 m (white body, colour stripe) and bus 12 × 2.5 m `#4A7BD0` (M14).

**Buildings** (M15, procedural along residential roads at street zoom):
- **Houses:** light green `#8FD16A` with darker green `#62A83E` gable roof faces.
- **Commercial:** blue `#4F7FD8` along primary and secondary roads.
- **Warehouses:** dark grey `#5A5A5A`.
- **Lots:** the ground lightened with `rgba(255,255,255,0.35)`, with white parking stripes.
- **Shadows:** down-right.

**UI:**
- **Chips:** white, 1 px `#D6D6D0` border, radius 6 px.
- **Text:** Lexend 500 (bundled via `@fontsource/lexend`) in `#1D1D1D`, with a subtle shadow.
- **Tooltip chips** look like `$60`, `Add lane` and `Elevation: 1`, and appear next to the cursor.
- An optional light screen-space darkening toward the bottom edge.

## 5. Quality gates (`scripts/check.sh`, CI, reviewer)

**Rust:**
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `[workspace.lints.clippy]`: `cognitive_complexity`, `too_many_lines`, `too_many_arguments`, `unwrap_used`, `expect_used`, `dbg_macro` and `todo` are all `deny`
- `clippy.toml` thresholds: complexity 10, lines 40, arguments 5
- Every crate uses `[lints] workspace = true`

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

**Comments:** `scripts/check-no-comments.mjs` rejects every comment in TS and Rust sources except `// SAFETY:`.

**Tests:**
- `cargo test` for everything
- `vitest` for pure TS modules
- Playwright smoke tests from M4 on, using `--use-angle=swiftshader`. They assert behaviour, not FPS.
  - **Browser:** locally, Playwright's default resolution finds `/opt/pw-browsers` because `PLAYWRIGHT_BROWSERS_PATH` is set. In CI, run `pnpm --dir web exec playwright install --with-deps chromium`.

**Versions:** pinned exactly. Fallbacks:
- ESLint 9 if `eslint-plugin-sonarjs` rejects ESLint 10.
- The newest TypeScript that `typescript-eslint` supports.
- Skip `wasm-opt` with a warning when it is missing.

## 6. Milestones

Each milestone ends with `scripts/check.sh` green, an approving review, a commit, and a push.

| # | Milestone | Acceptance |
|---|---|---|
| M0 | Tooling and scaffold: workspace, empty crates, Vite+TS, gates, `check.sh`, pinned wasm toolchain, `build-wasm.sh`, CI, CLAUDE.md rewritten to this plan | All gates pass. A deliberately over-complex Rust function and TS function each fail their gate, as do a comment in each language; then they are removed. The built page loads wasm in a worker and shows "Trapiks" |
| M1 | Map model and road derivation: `MapData`, mapgen reads Overpass JSON into roads, nodes, controls and turn restrictions | Fixture tests: splitting, chain merging, one-way variants, lane defaults and tags, layers, control assignment, restriction expansion, service/access filters, postcard round-trip, `map_hash` |
| M2 | Areas and the real map: multipolygon outer rings, coastline sea polygon, simplification, `fetch-osm.sh`, `build-map.sh`, committed map + LICENSE | Ring-assembly and coastline fixture tests. The real map is generated, and its stats (roads, nodes, points, bytes) are printed and meet §3.9. *Needs network access; if it is still blocked, M3 onward proceed on fixtures and a synthetic grid map made with mapgen `--synthetic`* |
| M3 | Network runtime: roads, links, lazy junctions, lane ranges, conflicts, priorities, signal clusters, spatial grid, region mask | Fixture tests: Phase-0 4-way with 2 lanes each way, T-junction, dual-carriageway cluster, one-way pair, dead-end U-turn. On the real map, every junction builds without panics, with timing reported |
| M4 | Roads slice: wasm `load`, road/node/area arrays across the boundary, worker fetch + gunzip, debug road render with basic pan and zoom | Playwright: the map renders, roads are counted on screen, and there are no console errors |
| M5 | Vehicles core: store, IDM, lane occupancy, leader lookup, spawn/despawn on fixtures, route arena, `state_hash` | Determinism: the same seed gives the same hash after 5,000 ticks, twice, and the fixture hash is recorded as a constant. IDM unit tests |
| M6 | Junction rules: entry rule, signals and amber, stop/yield, cluster spillback, timeout | Invariants on fixtures: no NaN, no overlap, no vehicle stuck for more than 300 s at low demand. Signal phase tests |
| M7 | Demand, routing and stats: A*, EMA costs, reroute budget, spawn queues, region mode, city and region stats, bench example | Determinism holds on the real map. The bench meets §3.9, including the delete-a-primary-road check |
| M8 | Vehicle pipeline: snapshots, double buffering, interpolation, speed control, stats messages, debug vehicle render | Playwright: vehicle count > 0 and the tick advances. The wasm `state_hash` after N ticks on the fixture equals the native constant |
| M9a | Renderer base: camera (pan, zoom, inertia, fly-to), LOD tiles, shared layered containers, road outlines and fills, areas | Playwright screenshot centred on a tile boundary shows no seam. City-zoom screenshot |
| M9b | Renderer detail: markings, one-way arrows, yield lines, elevated shadows, atlas vehicle particles, signal pills | Playwright screenshots at street zoom (EDSA–Ortigas and a skyway ramp), reviewed against the reference |
| M10 | Edit commands in sim-core: all §3.6 commands except roundabout and add-road, inverses, costs, budget, undo, id stability | Per-command apply → inverse round-trip tests (hash equality). Rejection tests |
| M11 | Editing UI: picking grid, hover and selection, tool palette, inspector, tooltip chips, `networkDelta` → tile rebuild | Playwright: delete a road and vehicles reroute; add a lane and the tile updates; undo |
| M12 | Challenges in sim-core: definitions, region metrics, baseline/evaluate phases, scoring | Scoring tests. Replaying a command log reproduces the same hash |
| M13 | Game screens: title, challenge list, sandbox, result screen, saves, traffic layer, attribution | Playwright plays one challenge end to end, and reload restores the save |
| M14 | Lane changes (mandatory + MOBIL) and vehicle kinds (car, jeepney, bus) | Lane-change safety (no overlap); mandatory changes reach the turn lane; determinism holds |
| M15 | Polish: procedural buildings, roundabout, add-road (straight + curved, elevated), inner rings, performance pass, GitHub Pages deploy workflow | Playwright covers the roundabout and add-road flows. `docs/perf.md` records the final numbers |

## 7. Risks

| Risk | Mitigation |
|---|---|
| Overpass is blocked | The user allows `overpass-api.de`. Until then, work proceeds on fixtures and a synthetic map. Fallback: a PBF reader for an uploaded extract |
| Gridlock in dense grids and dual carriageways | Cluster spillback, the entry timeout, capped challenge demand, and OSM turn restrictions |
| Signals mapped near junctions, not on them | Assign within 25 m and cluster within 30 m |
| Render cost of about 150k roads | LOD bands, one tile per road, lazy builds capped at 2 per frame, an LRU cache |
| Routing and reroute spikes | A* with an admissible heuristic, bounded trip lengths, and per-step reroute and spawn budgets |
| Challenge run time | Region subgraphs, and max speed sized to the wall-clock budget |
| wasm-bindgen CLI/crate mismatch | Pinned; `build-wasm.sh` checks this |
| Map file bloats git history | Regenerate rarely. Git LFS only if the file needs to change often |
| ODbL share-alike | Attribution in the game and a LICENSE next to the map |
