#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

readonly SOUTH=14.35
readonly WEST=120.90
readonly ROWS=4
readonly COLS=2
readonly LAT_STEP=0.11
readonly LON_STEP=0.12
readonly ENDPOINT=https://overpass-api.de/api/interpreter
readonly USER_AGENT="trapiks-mapgen/0.1 (https://github.com/jnebab/trapiks-game)"
readonly MAX_ATTEMPTS=60
readonly RETRY_DELAY=10

tile_bbox() {
  awk -v r="$1" -v c="$2" -v s="$SOUTH" -v w="$WEST" -v dl="$LAT_STEP" -v dw="$LON_STEP" \
    'BEGIN { printf "%.2f %.2f %.2f %.2f\n", s + r * dl, w + c * dw, s + (r + 1) * dl, w + (c + 1) * dw }'
}

query() {
  local s w n e
  read -r s w n e <<<"$(tile_bbox "$1" "$2")"
  cat <<QUERY
[out:json][timeout:900][maxsize:1073741824][bbox:${s},${w},${n},${e}];
area["ISO3166-2"="PH-00"]->.ncr;
(
  way["highway"~"^(motorway|trunk|primary|secondary|tertiary|unclassified|residential|living_street|road|service)(_link)?$"](area.ncr);
  node["highway"~"^(traffic_signals|stop|give_way)$"](area.ncr);
  relation["type"="restriction"](area.ncr);
  way["natural"="water"](area.ncr);
  way["waterway"="riverbank"](area.ncr);
  relation["natural"="water"](area.ncr);
  relation["waterway"="riverbank"](area.ncr);
  way["leisure"~"^(park|golf_course)$"](area.ncr);
  way["landuse"~"^(grass|recreation_ground|cemetery|forest|meadow)$"](area.ncr);
  relation["type"="multipolygon"]["leisure"~"^(park|golf_course)$"](area.ncr);
  relation["type"="multipolygon"]["landuse"~"^(grass|recreation_ground|cemetery|forest|meadow)$"](area.ncr);
  way["natural"="coastline"];
);
out body geom;
QUERY
}

fetch_tile() {
  local out="$3" attempt code
  for ((attempt = 1; attempt <= MAX_ATTEMPTS; attempt++)); do
    code=$(query "$1" "$2" | curl -sS -A "$USER_AGENT" -o "$out.part" -w "%{http_code}" --data-binary @- "$ENDPOINT" || true)
    if [[ "$code" == "200" ]]; then
      mv "$out.part" "$out"
      echo "$out: $(wc -c <"$out") bytes after $attempt attempts"
      return 0
    fi
    sleep "$RETRY_DELAY"
  done
  rm -f "$out.part"
  echo "$out: failed after $MAX_ATTEMPTS attempts" >&2
  return 1
}

if [[ "${1:-}" == "--print-query" ]]; then
  query "${2:?usage: fetch-osm.sh --print-query <row> <col>}" "${3:?usage: fetch-osm.sh --print-query <row> <col>}"
  exit 0
fi

mkdir -p data/osm
for ((row = 0; row < ROWS; row++)); do
  for ((col = 0; col < COLS; col++)); do
    out="data/osm/tile-${row}-${col}.json"
    [[ -s "$out" ]] && continue
    fetch_tile "$row" "$col" "$out"
  done
done
