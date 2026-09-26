#!/usr/bin/env bash

check_wasm_bindgen_version() {
  if ! command -v wasm-bindgen >/dev/null 2>&1; then
    echo "error: wasm-bindgen CLI not found on PATH; install it with: cargo install wasm-bindgen-cli" >&2
    return 1
  fi
  local expected actual
  expected=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')
  if [[ $(wc -l <<<"$expected") -ne 1 ]]; then
    echo "error: Cargo.lock contains more than one wasm-bindgen version: $(echo $expected)" >&2
    return 1
  fi
  actual=$(wasm-bindgen --version | awk '{print $2}')
  if [[ "$expected" != "$actual" ]]; then
    echo "error: wasm-bindgen CLI is $actual but Cargo.lock pins $expected; run: cargo install wasm-bindgen-cli --version $expected" >&2
    return 1
  fi
}
