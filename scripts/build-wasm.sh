#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "error: wasm-bindgen CLI not found on PATH; install it with: cargo install wasm-bindgen-cli" >&2
  exit 1
fi

expected=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')
if [[ $(wc -l <<<"$expected") -ne 1 ]]; then
  echo "error: Cargo.lock contains more than one wasm-bindgen version: $(echo $expected)" >&2
  exit 1
fi
actual=$(wasm-bindgen --version | awk '{print $2}')
if [[ "$expected" != "$actual" ]]; then
  echo "error: wasm-bindgen CLI is $actual but Cargo.lock pins $expected; run: cargo install wasm-bindgen-cli --version $expected" >&2
  exit 1
fi

cargo build -p trapiks-sim-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web/src/wasm/pkg target/wasm32-unknown-unknown/release/trapiks_sim_wasm.wasm

wasm_file=web/src/wasm/pkg/trapiks_sim_wasm_bg.wasm
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -O3 --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals --enable-reference-types --enable-multivalue \
    "$wasm_file" -o "$wasm_file"
else
  echo "warning: wasm-opt not found on PATH; skipping wasm optimisation" >&2
fi

TS_RS_EXPORT_DIR="$PWD/web/src/generated" cargo test -p trapiks-sim-core export_bindings
