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
  - `STREET_MIN_SCALE = 4`, which keeps 0.15 m dividers at 0.6 px or more
  - `CENTER_LINE = { color: 0xE2C23D, width: 0.18 }`
  - `LANE_DIVIDER = { color: 0x8E8E8E, width: 0.15, dash: 3, gap: 3 }`
  - `STOP_LINE = { color: 0x3A3A3A, width: 0.3, dash: 1, gap: 0.8 }`
  - `STOP_LINE_STOP_WIDTH = 0.5`
  - `SIGNAL_LINE_WIDTH = 0.4`
  - `ARROW = { color: 0x8E8E8E, length: 2.4, width: 1.2 }`

## Rust: render data from the network (`crates/sim-core/src/render/`)

`render.rs` becomes a module directory. Keep the M4 functions in `render/map.rs`.

### `setbacks.rs`

`pub fn road_setbacks(network: &Network) -> Vec<f32>` returns 2 values per road, `(setback at from, setback at to)`.
- At an end whose node has an active degree of 3 or more, the value is M3's `setback`. At any other end it is 0, so markings run straight through degree-2 nodes and only break at junction shapes.
- Deleted or inactive roads give `(0, 0)`.

### `junction_shape.rs`

`pub fn junction_shapes(network: &Network) -> JunctionShapes` covers every node with an active degree of at least 3:

- **Incident roads:** skip any road with `from == to`. For each other active incident road, take its departing link (leaving the node).
  - `t = min(setback + min(FILLET_RADIUS_MAX, w), length / 2)`, where `w` is the road's half-width.
  - `(c, d) = centre_pose(departing link, t)` gives the position and unit tangent at `t`. Using that point and tangent keeps curved roads correct.
  - Sort the roads by `libm::atan2(d.y, d.x)` with `f64::total_cmp` ascending, with ties broken by road id.
- **Edge points:** `p_left = c + perp_left(d)·w` and `p_right = c + perp_right(d)·w`. `perp_right(v) = (−v.y, v.x)` in y-down coordinates, and `perp_left` is its negation.
- **Fillets:** walk the roads in ascending order. For each consecutive pair `a`, then `b` (wrapping around), join `a.p_right` to `b.p_left` with a quadratic Bézier fillet.
  - The control point is the intersection of `a`'s right edge line (through `a.p_right`, direction `a.d`) and `b`'s left edge line (through `b.p_left`, direction `b.d`).
  - When the lines are parallel, or the intersection is farther from the node than either edge point, use the midpoint of the two edge points.
  - Add `QuadraticBezier { p0, p1, p2 }` with `point(t)` to `geom/bezier.rs`.
  - Sample each fillet into 6 segments (7 points).
- **Polygon:** the ring is the concatenation of the 7-point fillets, in order: 28 points for a 4-way. The straight road-end segments close it implicitly.
- **Output:** `JunctionShapes { node: Vec<u32>, layer: Vec<i8>, min_layer: Vec<i8>, ring_start: Vec<u32>, x: Vec<f32>, y: Vec<f32> }`, in node id order. `layer` is the maximum layer of the incident roads, and `min_layer` the minimum. M9c casts junction shadows only when `min_layer ≥ 1`.
- **Network helper:** `Network` gains `pub fn departing_link(road, node) -> LinkId`. It is geometric, with no lane filter, so one-way inbound roads are included too. Markers use `link_span(link).1` for `length − setback`.

### `approach_markers.rs`

`pub fn approach_markers(network: &Network) -> ApproachMarkers` covers every active link arriving at a node with an active degree of at least 3.

- **The line:** `(c, d) = centre_pose(link, length − setback)`. With `W` the road's total width and `n` the link's lane count, `(x1, y1) = c + perp_right(d)·W/2` and `(x2, y2) = c + perp_right(d)·(W/2 − n·3.2)`. This spans exactly the link's own lanes, including backward links, whose tangent `centre_pose` already reverses.
- **Kind** (`u8`):

  | Code | Name | When |
  |---|---|---|
  | 0 | none | Nothing is recorded |
  | 1 | yield (dashed) | A minor approach at a `Priority` or `Yield` node, meaning its class rank is below the node's highest incoming rank. This matches M6 |
  | 2 | stop (dashed, thicker) | A minor approach at a `Stop` node, or any approach at an `AllWayStop` node |
  | 3 | signal (solid) | `network.signal_state(link, 0).is_some()` |

  Internal cluster links give 0.
- **Output:** `ApproachMarkers { link: Vec<u32>, kind: Vec<u8>, x1, y1, x2, y2: Vec<f32> }`. Only non-zero kinds are included, in link order.


### Tests

- **`junction_shapes`:** on `four_way(2, 200.0)`, 1 shape with 28 points.
  - Every point lies within `setback + FILLET_RADIUS_MAX + 2w` of the node.
  - The ring does not self-intersect, checked with a brute-force segment test that skips adjacent segments.
  - The NE fillet's middle sample is farther from the node than the edge-line intersection (the sharp corner). The white area therefore covers the corner, which is the concave fillet. For `four_way(2, 200)`, the middle sample is (8.15, −8.15) from the node, against a corner at (6.4, −6.4).
- **`approach_markers`:**
  - `four_way` gives 4 signal markers.
  - `t_junction` gives exactly 1 yield marker, on link 4.
  - Each marker length equals the approach's lane count × 3.2, within 1e-3.
- **Determinism:** building twice gives equal output.

## wasm (`Engine`)

Add these getters, computed once at load:
- `roadSetbacks() -> Vec<f32>`
- `junctionShapes() -> JunctionShapeGeometry`, with columns `node`, `layer`, `minLayer`, `ringStart`, `x`, `y`
- `approachMarkers() -> ApproachMarkerGeometry`, with columns `link`, `kind`, `x1`, `y1`, `x2`, `y2`

The worker adds them to `ready` (transferred), and the protocol and guards grow to match.

## TS rendering

- **Layer containers:** `layers.ts` adds a `markings` pass after `fill` for every layer. The markings containers are visible only while `scale ≥ STREET_MIN_SCALE`.
- **Tile index:** a detail tile also owns the junction shapes and approach markers whose node or marker midpoint lies in it. `TileEntry` gains `junctions: Uint32Array` and `markers: Uint32Array`, and its bounds include them.
- **`tile-builder.ts`**, for each tile:
  - **Junction shapes:** filled white in `layers.road(layer, 'fill')`, in the same `Graphics` as the tile's fills, after the road strokes.
    - Each 7-point fillet is also stroked as an open polyline in the `outline` pass, at twice the edge thickness: 0.7 m on the ground, 1.0 m when elevated. The half outside the polygon shows as the edge.
    - The straight segments across road ends are never stroked, so a mixed-layer node never draws a bar across a lower road.
  - **Markings** are built in the same `update` that first shows the tile at street scale, into `layers.road(layer, 'markings')`:
    - **Centre line:** two-way roads get a solid yellow line at right-offset `W/2 − F·3.2` from the centreline in the forward frame, where `W = (F + B)·3.2`.
    - **Lane dividers:** dashed lines at right-offsets `W/2 − k·3.2` for `k = 1 … F−1`, and at `−(W/2 − k·3.2)` for `k = 1 … B−1`.
    - **One-way arrows:** one every `ARROW_SPACING` along one-way roads, starting at `setback_from + 20`, as a small filled triangle on the centreline.
      - The direction comes from whichever lane count is non-zero: forward when `F > 0`, else backward. Edits then cannot flip arrows silently.
      - A trimmed span too short for the first arrow gets one arrow at its midpoint.
    - Every marking is trimmed to `[setback_from, length − setback_to]` along the road.
    - **Approach markers:** dashed (kinds 1 and 2) or solid (kind 3) lines, in the markings container of the layer of the marker's road. Widths are `STOP_LINE.width`, `STOP_LINE_STOP_WIDTH` and `SIGNAL_LINE_WIDTH`.
- **Overlay:** `hud/debug-overlay.ts` adds `data-markings-built`, the number of visible tiles whose markings exist.
- **Later edits:** M11's `networkDelta` must recompute setbacks, shapes and markers for the nodes it touches.
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
- **Playwright** (`e2e/street-detail.spec.ts`), at `/?map=synthetic&vph=0&cx=750&cy=750&z=12`, with the default 1280 × 720 viewport, an arterial–arterial signalized junction:
  - Wait until `data-markings-built === data-tiles-visible`, with both above 0.
  - The yellow centre line at world (780, 750) is screen (1000, 360). The line is 2.16 px wide, centred on y = 360. Sample rows 359 and 360 at x = 1000: R > 200, G > 170, B < 120.
  - The fillet at world (757.3, 742.7) is screen (727, 272). It must be white; without the fillet it would be ground colour.
  - Save the screenshot `street-detail.png`.
  - Also save `street-residential.png` at `cx=150&cy=750&z=12`, where a residential street meets an arterial. The residential approaches are minor, so it should show yield lines.

## Acceptance

- `bash scripts/check.sh` passes.
- Report the screenshots. The orchestrator compares them with the reference.
- No comments. Do not modify `CLAUDE.md`, `docs/` or `.claude/`.
