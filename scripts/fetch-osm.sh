#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

readonly SOUTH=14.35
readonly WEST=120.90
readonly NORTH=14.79
readonly EAST=121.14
readonly MID_LAT=14.57
readonly MID_LON=121.02
readonly ENDPOINT=https://overpass-api.de/api/interpreter

quadrant_bbox() {
  case "$1" in
    1) echo "$MID_LAT $WEST $NORTH $MID_LON" ;;
    2) echo "$MID_LAT $MID_LON $NORTH $EAST" ;;
    3) echo "$SOUTH $WEST $MID_LAT $MID_LON" ;;
    4) echo "$SOUTH $MID_LON $MID_LAT $EAST" ;;
    *) echo "unknown quadrant: $1" >&2; exit 2 ;;
  esac
}

query() {
  local s w n e
  read -r s w n e <<<"$(quadrant_bbox "$1")"
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

if [[ "${1:-}" == "--print-query" ]]; then
  query "${2:?usage: fetch-osm.sh --print-query <1-4>}"
  exit 0
fi

mkdir -p data/osm
for quadrant in 1 2 3 4; do
  out="data/osm/q${quadrant}.json"
  query "$quadrant" | curl --fail --retry 5 --retry-all-errors --retry-delay 30 -o "$out" --data-binary @- "$ENDPOINT"
done
