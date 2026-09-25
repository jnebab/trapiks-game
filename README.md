# Trapiks

A browser puzzle game about fixing the real Metro Manila road network. The simulation is written in Rust and compiled to WebAssembly; the game runs in TypeScript with PixiJS.

## Prerequisites

- Rust 1.94.1 with clippy, rustfmt and the `wasm32-unknown-unknown` target
- `wasm-bindgen-cli` 0.2.129
- binaryen (`wasm-opt`) version 132
- Node 22 and pnpm 10.33.0

## Commands

```bash
scripts/check.sh                 # every gate: fmt, clippy, tests, wasm build, typecheck, lint, format, vitest, comments
cargo test -p trapiks-sim-core
./scripts/build-wasm.sh
pnpm --dir web dev
```
