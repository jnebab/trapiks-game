# Trapiks

Browser puzzle game: the player fixes simplified, real-inspired Metro Manila roads, highways and intersections to improve traffic. Top-down 2D. Each level is a hand-built chokepoint (e.g. EDSA–Ortigas, Magallanes, Balintawak). The player gets a budget and a limited toolset, and a microscopic traffic sim scores the result (average commute time / throughput vs. target).

The core loop is **fix, don't build**: levels are pre-authored networks the player modifies. There is no freeform city growth.

## Stack

- **Simulation:** Rust.
  - `trapiks-sim-core` is pure Rust with no wasm/web dependencies.
  - `trapiks-sim-wasm` is a thin `wasm-bindgen` layer over it.
- **Web:** TypeScript (strict), Vite, PixiJS renderer. The sim runs inside a Web Worker.
- **HUD framework:** not chosen yet. Phase 0 uses plain DOM for debug overlays. Don't introduce a UI framework without asking.
- **Backend:** none for MVP; the game is hosted as a static site.
  - Later: FastAPI + SQLModel.
  - Later: `trapiks-sim-py` (PyO3) to re-run replays and verify leaderboard scores.
- **Shared types:** generated from Rust to TS with `ts-rs`. Never hand-write duplicates.

## Layout

```
trapiks/
├─ Cargo.toml              # workspace
├─ crates/
│  ├─ sim-core/            # trapiks-sim-core
│  └─ sim-wasm/            # trapiks-sim-wasm
├─ scripts/build-wasm.sh   # cargo build → wasm-bindgen --target web → wasm-opt
└─ web/                    # Vite + TS
   └─ src/
      ├─ main.ts           # PixiJS renderer + debug overlay
      ├─ sim.worker.ts     # owns the wasm module, fixed-timestep loop
      └─ wasm/pkg/         # generated, gitignored
```

## Runtime architecture

1. **Main thread:** PixiJS rendering and HUD only. It never runs sim logic.
2. **Worker:** loads the wasm module and runs a fixed-timestep loop (10–20 Hz). It posts a state snapshot after each step.
3. **Renderer:** interpolates between the last two snapshots at display refresh rate.

Rules at the JS/wasm boundary:

- Call into wasm **once per tick** (`step(dt)`), never once per vehicle.
- Player edits go into a command queue. The queue is applied at the start of the next `step`.
- Vehicle state is stored as struct-of-arrays in wasm memory.
  - The worker reads it through typed-array views.
  - The worker then copies it into a **transferable** buffer for the main thread.
- Views over wasm memory detach if the memory grows.
  - Preallocate vehicle capacity at level load.
  - Recreate the views after any growth.
- The sim is single-threaded for now. Don't use `SharedArrayBuffer`, so no COOP/COEP headers are required.

## Sim conventions

- **Data layout:** struct-of-arrays. Refer to entities by integer IDs (`u32` indices into `Vec`s), never by references. This keeps the borrow checker out of the way.
- **Driving models:** IDM for car-following. MOBIL for lane changes, which comes after Phase 0.
- **Units:** SI internally (meters, seconds, m/s).
- **Determinism (non-negotiable).** The same seed plus the same commands must produce the same state on every platform.
  - Use a seeded PRNG passed in at level load. No `thread_rng`, no system time.
  - Use the `libm` crate for transcendental functions.
  - Never let results depend on `HashMap`/`HashSet` iteration order, which is randomized in Rust. Use `Vec`, `BTreeMap`, or sorted IDs.

## Commands

```bash
cargo test -p trapiks-sim-core
./scripts/build-wasm.sh
pnpm --dir web dev
```

## Gotchas

- The `wasm-bindgen` CLI version must exactly match the `wasm-bindgen` crate version in `Cargo.lock`. Pin both.
- Required toolchain: `rustup target add wasm32-unknown-unknown`, `wasm-bindgen-cli`, and binaryen (for `wasm-opt`).

## Phase 0 scope

Goal: prove the sim model and the worker/wasm pipeline.

**In scope:**

- One signalized 4-way intersection with 2 lanes each way and a fixed-time signal.
- Vehicles spawn at the edges using the seeded RNG. Each is assigned a turn movement at spawn.
- IDM following, stopping at red, clearing the box on green.
- A headless test: the same seed must produce an identical state hash after N ticks.
- A debug render in PixiJS: road rects, vehicle rects, signal state, plus an FPS and vehicle-count overlay.

**Out of scope:** routing, lane changes, level loading, art, and the HUD framework.

## Working agreements

- Propose a plan before large changes.
- Keep commits small and logical.
- Ask before pushing.
- Never commit secrets or `.env` files.
- Keep `sim-core` free of wasm/web dependencies so it stays testable natively.
