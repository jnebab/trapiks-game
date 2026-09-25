# M9a: renderer base (camera motion, LOD tiles, layered road drawing, areas)

Goal: replace M4's single-`Graphics` debug drawing with the real renderer structure:
- Tiled, level-of-detail road drawing with seamless junctions and seamless tile borders.
- Elevated layers drawn above ground.
- Water and park areas.
- Camera inertia and fly-to.

Markings, arrows, shadows, vehicle atlas and signal pills are M9b.

Follow `docs/plan.md` §3.8 and §4. The reference screenshots are at `/tmp/claude-0/-home-user-trapiks-game/74a64fa3-bf2f-5a5c-88ae-f83b4d67c7ec/scratchpad/style-ref/`. Look at them before starting.

## Decisions (fixed)

- **Two tile bands**, not three:
  - `city`: 4,096 m tiles holding only roads with class rank ≥ Primary.
  - `detail`: 512 m tiles holding every road.
  - The plan's district and street bands share the `detail` tiles; M9b's markings are shown only at street scale.
- **Scale thresholds** (px per metre):
  - `scale < CITY_MAX_SCALE` (0.35): only city tiles are shown.
  - Otherwise only detail tiles are shown.
  - M9b adds the street-detail threshold. Don't define unused constants now.
- **Tile ownership:** each road belongs to exactly one tile per band, the tile containing the centre of its bbox. A tile's cull bounds are the union of its roads' bboxes.
- **Road style:**
  - Surface white `#FFFFFF`, width `total lanes × 3.2` m.
  - Ground edge `#9B9B97`, drawn as an outline stroke of `width + 0.7` m.
  - Elevated edge (layer > 0) `#151515`, drawn as an outline of `width + 1.0` m.
  - Tunnels (layer < 0) at container alpha 0.4.
  - Round joins and round caps for both outline and fill. The round caps fill junction corners, and all outlines sit beneath all fills.
- **City-band style:** fill only, white, with exaggerated widths by rank so major roads read at 0.02–0.35 px/m:
  - rank ≥ 13: 60 m
  - rank ≥ 11: 45 m
  - otherwise: 35 m
  - Each also gets a `#9B9B97` outline 20 % wider.
- **Class ranks:** the TS side needs them. `MapMeta` gains `class_ranks: Vec<u8>` (indexed by class code, from `RoadClass::rank`), so TS never hand-writes the table.

## Layer containers (`render/layers.ts`)

- `createLayers(world) → Layers` creates these containers under `world`, in order:
  1. `areas`
  2. For each layer value from −3 to 5, ascending: `outline` and `fill`
  3. `vehicles`, which M8's `ParticleContainer` moves into
  4. `overlay`
- The layer −3 to −1 containers get `alpha = 0.4`.
- `Layers.road(layer, pass)` returns the container for a road layer (clamped to −3..5) and a pass (`'outline' | 'fill'`).
- M9b will add `shadow` before `outline` and `markings` after `fill`, following the same pattern.

## Tiles (`render/tiles/`)

| File | Content |
|---|---|
| `tile-key.ts` | Pure: `tileKey(band, tx, ty): string` and `tileOf(size, x, y): [tx, ty]` |
| `tile-index.ts` | Pure: `buildTileIndex(roads: RoadArrays, band: Band): TileIndex`. `Band` is `{ name: 'city' \| 'detail'; size: number; include(road): boolean }`. `TileIndex` is a `Map<string, TileEntry>`, where `TileEntry { roads: Uint32Array; bounds: Rect }`. Roads are grouped by the tile of their bbox centre, road ids are ascending, and `bounds` is the union of the road bboxes |
| `visible.ts` | Pure: `visibleTiles(index, view: Rect, margin: number): string[]`. Iterates every entry and keeps those whose bounds intersect `view` expanded by `margin`, ordered by distance from the view centre |
| `tile-builder.ts` | Builds one tile: `buildTile(entry, roads, style): TileGraphics`. `TileGraphics` holds `{ layer, pass, graphics }` pieces. Roads are grouped by `(layer, width)`. Each group becomes one `Graphics` per pass, with a `moveTo`/`lineTo` subpath for every road, then a single `stroke()` |
| `tile-cache.ts` | `class TileCache`: an LRU with a capacity of 256 tiles per band. It uses a `Map` in insertion order: on access, delete then re-set; on eviction, `destroy()` the pieces' graphics and remove them from their containers. `show(keys)` makes those tiles visible and hides the rest. `pending(keys)` returns the keys not yet built |
| `tile-manager.ts` | `class TileManager`: owns both bands' indexes and caches. Each frame, `update(camera, viewW, viewH)` does the following. It picks the band by scale and computes the visible keys with a 256 m margin. It builds at most `BUILDS_PER_FRAME = 2` pending tiles, nearest first, adding each piece to `layers.road(layer, pass)`. It shows the visible tiles of the active band and hides every tile of the other band |

- **Rect:** `{ minX, minY, maxX, maxY }`, in `render/rect.ts`, with `intersects` and `expand`.

## Areas (`render/areas.ts`)

- `drawAreas(areas: AreaArrays, kindNames, container)` draws one `Graphics` holding every ring: water `#A8C0F4` and parks `#CDE6A6`, both via `poly(points).fill(color)`, with no outline.
- It is drawn once at load.

## Camera motion (`render/camera-motion.ts`)

This is pure math, with vitest coverage.

- **Inertia:**
  - `releaseVelocity(samples: {t, x, y}[]): {vx, vy}` averages the pointer samples from the last 100 ms, in px/ms.
  - `stepInertia(camera, velocity, dtMs): {camera, velocity}` applies `pan(v·dt)` and decays the velocity by `0.92^(dt/16)`. It stops below 0.01 px/ms.
- **Fly-to:**
  - `flyTo(from: Camera, target: {x, y, scale}, viewW, viewH, durationMs) → (tMs) => Camera`.
  - Uses ease-in-out cubic on `t / duration`.
  - Interpolates the world centre linearly and the scale in log space.
  - At `t ≥ duration`, it returns the exact target.
- `render/camera-input.ts` uses these: inertia after a drag release, cancelled by a new pointer-down or wheel.

## Main-thread wiring

- **`render/map-view.ts`:** `createMapView(app, ready: ReadyMessage): MapView`.
  - Builds `layers`, draws the areas and creates the `TileManager`.
  - Registers a ticker callback that runs `tileManager.update` with the current camera.
  - Exposes `flyTo(x, y, scale)`.
- `app/debug-scene.ts` uses `createMapView` instead of the M4 debug drawing.
- Delete `render/debug-roads.ts` and `render/debug-areas.ts`.
- The Rust side adds `class_ranks` to `MapMeta` in `sim-core/src/render.rs`, with a test that it matches `RoadClass::rank` for every code.
- **`hud/debug-overlay.ts`:** add `tiles {built}/{visible}` and a `data-band` attribute (`city` or `detail`).
- **Test hooks:** `?cx=<x>&cy=<y>&z=<scale>` sets the initial camera (world centre and px/m) instead of fitting the bounds. Clamp the values to the map bounds and the scale limits.

## Tests

- **vitest:**
  - `tile-index.test.ts`, on a hand-built `RoadArrays` of 5 roads:
    - Every road is in exactly one tile.
    - A road whose bbox centre lies in tile (1, 0) but which extends into (0, 0) belongs to (1, 0), and that tile's bounds cover the extension.
    - The city band excludes a residential road.
  - `visible.test.ts`: the margin, and ordering by distance.
  - `camera-motion.test.ts`: inertia decays to a stop; fly-to hits both endpoints exactly; the scale midpoint is the geometric mean.
  - `rect.test.ts`.
- **Playwright** (`e2e/renderer.spec.ts`), on the synthetic map:
  1. `/?map=synthetic&cx=1024&cy=450&z=6`. The camera is centred where the horizontal residential road at y = 450, the block from x = 900 to 1050, crosses the x = 1024 tile boundary. That road's bbox centre (975) is in tile 1.
     - Wait until `data-band` is `detail` and tiles have finished building (the overlay built count equals the visible count).
     - Sample canvas pixels at the road centre line, 3 px left and 3 px right of the boundary's screen x. Both must be white (every channel ≥ 245). Read pixels with `page.screenshot` and decode the PNG with `pngjs`, pinned as a dev dependency.
     - Save the screenshot `renderer-boundary.png`.
  2. `/?map=synthetic&z=0.1`: `data-band` is `city`, and the screenshot `renderer-city.png` is saved.
  3. No console errors.

## Acceptance

- `bash scripts/check.sh` passes.
- Report both screenshot paths. The orchestrator compares them with the reference.
- Report built tiles and frame time at district zoom on the synthetic map, from the overlay FPS.
- No comments. Do not modify `CLAUDE.md`, `docs/` or `.claude/`.
