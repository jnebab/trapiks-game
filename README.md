# Trapiks

A browser puzzle game about fixing the real Metro Manila road network. The simulation is written in Rust and compiled to WebAssembly; the game runs in TypeScript with PixiJS.

## Play

The game is a static site deployed to GitHub Pages by `.github/workflows/pages.yml` on every push to the default branch (or by running the workflow by hand). Open the Pages URL of this repository, pick a challenge or the sandbox, and fix the traffic. Add `?map=synthetic` to try the small synthetic city.

## Develop

### Prerequisites

- Rust 1.94.1 with clippy, rustfmt and the `wasm32-unknown-unknown` target
- `wasm-bindgen-cli` 0.2.129
- binaryen (`wasm-opt`) version 132
- Node 22 and pnpm 10.33.0

### Commands

```bash
scripts/check.sh                 # every gate: fmt, clippy, tests, wasm build, typecheck, lint, format, vitest, comments
cargo test -p trapiks-sim-core
./scripts/build-wasm.sh
pnpm --dir web dev
```

To build for a sub-path, as Pages does, set the base path:

```bash
TRAPIKS_BASE=/trapiks-game/ pnpm --dir web build
```

Performance runs are manual: generate `web/public/maps/grid120.bin.gz` with `cargo run --release -p trapiks-mapgen -- --out web/public/maps/grid120.bin.gz --synthetic 120x120@150`, then run `TRAPIKS_PERF=1 pnpm --dir web exec playwright test e2e/perf --workers=1`. Results are appended to `web/test-results/perf.jsonl`.
