# Trapiks

Browser puzzle game. The player fixes the real Metro Manila road network, imported from OpenStreetMap: every drivable street, plus highways, expressways, skyways and intersections. The game is top-down 2D, styled after Trafficity. A microscopic traffic sim scores the result.

The core loop is **fix, don't build**. The network already exists, and the player deletes and modifies roads and junctions. Modifications include:
- lanes, direction, speed and elevation
- intersection control, signal timing and turn bans
- flyovers and roundabouts
- short connector roads

There are two modes:
- **Challenges:** named chokepoints such as EDSA–Ortigas, Magallanes and Balintawak. Each runs on a region subgraph, with a budget and a target.
- **Sandbox:** the whole city.

The full design, fixed sim decisions and milestones are in `docs/plan.md`. Per-milestone specs are in `docs/milestones/`.

## Stack

- **Simulation:** Rust.
  - `trapiks-sim-core` is pure Rust with no wasm/web dependencies.
  - `trapiks-sim-wasm` is a thin `wasm-bindgen` layer over it.
- **Map pipeline:** `trapiks-mapgen`, a native CLI that converts OSM (Overpass JSON) into the committed `web/public/maps/metro-manila.bin.gz`.
- **Web:** TypeScript (strict), Vite, PixiJS 8. The sim runs inside a Web Worker.
- **HUD:** plain DOM and CSS modules written in TypeScript. No UI framework, and no React.
- **Backend:** none; the game is hosted as a static site.
- **Out of scope:** a server backend, leaderboards, and Python bindings. Don't build toward them.
- **Shared types:** generated from Rust to TS with `ts-rs` into `web/src/generated/`. Never hand-write duplicates.

## Layout

```
Cargo.toml                 workspace
crates/
├─ sim-core/               trapiks-sim-core
├─ sim-wasm/               trapiks-sim-wasm
└─ mapgen/                 trapiks-mapgen
scripts/                   check.sh, build-wasm.sh, check-no-comments.mjs, fetch-osm.sh, build-map.sh
web/
├─ public/maps/            committed game map + ODbL LICENSE
└─ src/
   ├─ main.ts              bootstrap only
   ├─ app/                 screens and game flow
   ├─ sim/                 worker client, protocol, interpolation
   ├─ worker/              sim.worker.ts, fixed-timestep loop
   ├─ render/              PixiJS renderer
   ├─ edit/                tools, picking, selection
   ├─ hud/                 DOM widgets + CSS
   ├─ generated/           ts-rs output (gitignored)
   └─ wasm/pkg/            wasm-bindgen output (gitignored)
docs/                      plan.md, milestones/, perf.md
```

## Runtime architecture

1. **Main thread:** PixiJS rendering, HUD, input and picking. It never runs sim logic.
2. **Worker:** fetches and gunzips the map, loads the wasm module, and runs a fixed-timestep loop (`dt = 0.1 s`). It posts a snapshot after each 100 ms wall tick.
3. **Renderer:** interpolates between the last two snapshots at display refresh rate, matching vehicles by id.

Rules at the JS/wasm boundary:

- Call into wasm **once per tick** (`step`), never once per vehicle.
- Player edits go into a command queue. The queue is applied at the start of the next `step`.
- Vehicle state is stored as struct-of-arrays in wasm memory.
  - The worker reads it through typed-array views.
  - The worker then copies it into **transferable** buffers for the main thread.
- Views over wasm memory detach if the memory grows.
  - Preallocate vehicle capacity at load.
  - Recreate the views whenever `memory.buffer` changes.
- The sim is single-threaded. Don't use `SharedArrayBuffer`, so no COOP/COEP headers are required.

## Sim conventions

- **Data layout:** struct-of-arrays. Refer to entities by integer IDs (`u32` indices into `Vec`s), never by references.
- **Stable IDs:** roads and nodes are never removed. Deletion sets a flag, and additions append. Only undoing an appending command truncates the ids it appended.
- **Driving models:** IDM for car-following. Mandatory lane changes plus MOBIL come in M14.
- **Units:** SI internally (meters, seconds, m/s). Traffic drives on the right.
- **Determinism (non-negotiable).** The same map, seed and commands must produce the same state hash on every platform, native and wasm.
  - Use a seeded PCG32 threaded explicitly. No `thread_rng`, no system time.
  - Use the `libm` crate for transcendental functions.
  - Never let results depend on `HashMap`/`HashSet` iteration order. Use `Vec`, `BTreeMap`, or sorted IDs.
  - Break ties by id.

## Code standards

- **No comments** in TS or Rust sources. The one exception is `// SAFETY:` on `unsafe` blocks. `scripts/check-no-comments.mjs` enforces this.
- Keep cognitive complexity low: small single-purpose functions, early returns, and shallow nesting. Clippy and ESLint enforce the thresholds.
- One concept per module. Keep files short. Prefer plain data and functions over class hierarchies.
- No dead code, speculative abstractions, TODOs or `unwrap`/`expect` in library code.

## Commands

```bash
scripts/check.sh                 # every gate: fmt, clippy, tests, wasm build, typecheck, lint, format, vitest, comments
cargo test -p trapiks-sim-core
./scripts/build-wasm.sh
pnpm --dir web dev
```

## Gotchas

- The `wasm-bindgen` CLI version must exactly match the `wasm-bindgen` crate version in `Cargo.lock`. Both are pinned to 0.2.129.
- Required toolchain:
  - `rustup target add wasm32-unknown-unknown`
  - `wasm-bindgen-cli` 0.2.129
  - binaryen (`wasm-opt`) version 132
- OSM data is licensed under ODbL. Keep the attribution in the game and the LICENSE next to the map.

## Working agreements

- **Roles:**
  - The main session orchestrates.
  - `.claude/agents/implementer.md` implements.
  - `.claude/agents/reviewer.md` reviews every plan and every diff.
- Propose a plan before large changes.
- Keep commits small and logical.
- Push each reviewed milestone to the working branch.
- Never commit secrets or `.env` files.
- Keep `sim-core` free of wasm/web dependencies so it stays testable natively.
