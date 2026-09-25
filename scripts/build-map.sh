#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

mkdir -p web/public/maps
if [[ "${1:-}" == "--synthetic" ]]; then
  cargo run --release -p trapiks-mapgen -- --out web/public/maps/synthetic.bin.gz --synthetic 30x30@150
  exit 0
fi
cargo run --release -p trapiks-mapgen -- --out web/public/maps/metro-manila.bin.gz data/osm/q*.json
