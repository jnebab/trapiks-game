#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir web install --frozen-lockfile
pnpm --dir web build
pnpm --dir web typecheck
pnpm --dir web lint
pnpm --dir web format
pnpm --dir web test
pnpm --dir web check:comments

echo "ALL CHECKS PASSED"
