# M9b: street detail (junction shapes, markings, stop lines, arrows)

Goal: at street zoom, the map reads like the Trafficity reference:
- junction areas with concave rounded corners (fillets)
- a yellow centre line on two-way roads
- dashed grey lane dividers
- dashed yield and stop lines at junction entries
- grey one-way arrows

Everything tied to junction geometry is computed in Rust from the same `Network` the sim uses, so markings line up with where vehicles actually stop.

Elevated shadows, signal pills and vehicle polish are M9c.

Look at the reference screenshots in `/tmp/claude-0/-home-user-trapiks-game/74a64fa3-bf2f-5a5c-88ae-f83b4d67c7ec/scratchpad/style-ref/` before starting.

## Constants

- **Rust, `render` module:** `FILLET_RADIUS_MAX = 6.0`, `ARROW_SPACING = 40.0`.
- **TS, `render/style.ts`:**
  - `STREET_MIN_SCALE = 2.5`
  - `CENTER_LINE = { color: 0xE2C23D, width: 0.18 }`
  - `LANE_DIVIDER = { color: 0x8E8E8E, width: 0.15, dash: 3, gap: 3 }`
  - `STOP_LINE = { color: 0x3A3A3A, width: 0.3, dash: 1, gap: 0.8 }`
  - `ARROW = { color: 0x8E8E8E, length: 2.4, width: 1.2 }`

## Rust: render data from the network (`crates/sim-core/src/render/`)

`render.rs` becomes a module directory. Keep the M4 functions in `render/map.rs`.

### `setbacks.rs`

`pub fn road_setbacks(network: &Network) -> Vec<f32>` returns 2 values per road, `(setback at from, setback at to)`, using M3's `setback`. Deleted or inactive roads give `(0, 0)`.

### `junction_shape.rs`

`pub fn junction_shapes(network: &Network) -> JunctionShapes` covers every node with an active degree of at least 3:

- **Incident roads:** for each active incident road, `d` is its unit direction leaving the node, taken from the tangent at the setback point. `w` is its half-width. Sort the roads by `atan2(d.y, d.x)` ascending, with ties broken by road id.
- **Corner points:** for each road, its two edge points at the setback distance are `p_left = node + d·setback + perp_left(d)·w` and `p_right = node + d·setback + perp_right(d)·w`. `perp_left` is the negation of `perp_right`.
- **Ring:** walk the roads around the node. For consecutive roads `a`, then `b` (wrapping around), connect `a`'s left edge point to `b`'s right edge point with a quadratic Bézier fillet.
  - The control point is the intersection of `a`'s left edge line and `b`'s right edge line. When the lines are parallel, or the intersection lies more than `FILLET_RADIUS_MAX + max(wa, wb)` from the node, use the midpoint of the two edge points instead.
  - Sample each fillet at 6 segments.
- **Polygon:** the ring is `[a.p_right, a.p_left, fillet(a→b)…, b.p_right, b.p_left, …]`, closed.
- **Output:** `JunctionShapes { node: Vec<u32>, layer: Vec<i8>, ring_start: Vec<u32>, x: Vec<f32>, y: Vec<f32> }`, where `layer` is the maximum layer of the incident roads, in node id order.

### `approach_markers.rs`

`pub fn approach_markers(network: &Network) -> ApproachMarkers` covers every active link arriving at a node with an active degree of at least 3.

- **The line:** a segment across the link's lanes at `s = length − setback`, from the lane-0 outer edge to the last lane's inner edge, using M3's `link_pose` and lane offsets.
- **Kind** (`u8`):

  | Code | Name | When |
  |---|---|---|
  | 0 | none | Nothing is recorded |
  | 1 | yield (dashed) | A minor approach at a `Priority` node, meaning its class rank is below the node's highest incoming rank, or any approach at a `Yield` node |
  | 2 | stop (dashed, thicker) | A minor approach at a `Stop` node, or any approach at an `AllWayStop` node |
  | 3 | signal (solid) | A signal-cluster approach link |

  Internal cluster links give 0.
- **Output:** `ApproachMarkers { link: Vec<u32>, kind: Vec<u8>, x1, y1, x2, y2: Vec<f32> }`. Only non-zero kinds are included, in link order.

### `road_ends.rs`

`RoadRender` gains `from: Vec<u32>` and `to: Vec<u32>`.

### Tests

- **`junction_shapes`:** on `four_way(2, 200.0)`, 1 shape with `4 × (2 + 7)` points. Every point lies within `setback + FILLET_RADIUS_MAX + w` of the node. The ring is non-self-intersecting (brute-force segment test).
- **`approach_markers`:**
  - `four_way` gives 4 signal markers.
  - `t_junction` gives exactly 1 yield marker, on link 4.
  - Each marker length equals the approach's lane count × 3.2, within 1e-3.
- **Determinism:** building twice gives equal output.

## wasm (`Engine`)

Add these getters, computed once at load:
- `roadSetbacks() -> Vec<f32>`
- `junctionShapes() -> JunctionShapeGeometry`, with columns `node`, `layer`, `ringStart`, `x`, `y`
- `approachMarkers() -> ApproachMarkerGeometry`, with columns `link`, `kind`, `x1`, `y1`, `x2`, `y2`
- `RoadGeometry` gains `from` and `to`

The worker adds them to `ready` (transferred), and the protocol and guards grow to match.

## TS rendering

- **Layer containers:** `layers.ts` adds a `markings` pass after `fill` for every layer. The markings containers are visible only while `scale ≥ STREET_MIN_SCALE`.
- **Tile index:** a detail tile also owns the junction shapes and approach markers whose node or marker midpoint lies in it. `TileEntry` gains `junctions: Uint32Array` and `markers: Uint32Array`, and its bounds include them.
- **`tile-builder.ts`**, for each tile:
  - **Junction shapes:** filled white in `layers.road(layer, 'fill')`, in the same `Graphics` as the tile's fills, after the road strokes. Their rings are also stroked in the `outline` pass with the same edge colour and width as that layer's roads, so the fillet edges are visible.
  - **Markings,** built lazily the first time the tile is shown at street scale, into `layers.road(layer, 'markings')`:
    - **Centre line:** two-way roads get a solid yellow line at right-offset `W/2 − F·3.2` from the centreline in the forward frame, where `W = (F + B)·3.2`.
    - **Lane dividers:** dashed lines at right-offsets `W/2 − k·3.2` for `k` in `1..F`, and at `−(W/2 − k·3.2)` for `k` in `1..B`.
    - **One-way arrows:** one every `ARROW_SPACING` along one-way roads, starting at `setback_from + 20`, as a small filled triangle pointing in the travel direction on the centreline.
    - Every marking is trimmed to `[setback_from, length − setback_to]` along the road.
    - **Approach markers:** dashed (kinds 1 and 2) or solid (kind 3) lines, in the markings container of the layer of the marker's road.
- **`render/polyline.ts`** (pure, vitest-covered):
  - `cumulative(points)`
  - `slice(points, from, to)`: the sub-polyline between arc lengths
  - `offset(points, d)`: per-vertex offset along the averaged normal, with a miter limit of 2, using `perp_right = (−y, x)` in y-down coordinates
  - `dashes(points, dash, gap) -> segments`
  - `arrowsAlong(points, spacing, start) -> {x, y, angle}[]`

## Tests

- **vitest:**
  - `polyline.test.ts`: offset of a straight line and of a right angle, including the miter limit; dash counts on a 30 m line with 3/3 giving 5 dashes; slice at the bounds; arrow angles.
  - `markings.test.ts`: for a 2+2 road, the centre-line offset is 0 and there is one divider per direction at ±3.2. For a 3+1 road, the centre-line offset is `W/2 − 9.6 = −3.2`.
- **Playwright** (`e2e/street-detail.spec.ts`), at `/?map=synthetic&vph=0&cx=750&cy=750&z=12`, an arterial–arterial signalized junction:
  - Wait for the tiles.
  - Sample a pixel on the yellow centre line of the arterial 30 m from the junction: R > 200, G > 170, B < 120.
  - Sample the junction centre: white.
  - Save the screenshot `street-detail.png`.
  - Also save `street-residential.png` at `cx=150&cy=750&z=12`, where a residential street meets an arterial. The residential approaches are minor, so it should show yield lines.

## Acceptance

- `bash scripts/check.sh` passes.
- Report the screenshots. The orchestrator compares them with the reference.
- No comments. Do not modify `CLAUDE.md`, `docs/` or `.claude/`.
