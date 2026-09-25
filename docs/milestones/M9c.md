# M9c: depth and life (elevated shadows, live signal pills, vehicle sprite)

Goal: finish the Trafficity look:
- Elevated roads cast soft shadows down-right onto the ground and onto lower roads.
- Signalized approaches show small pills in the live signal colour.
- Cars get a polished sprite.
- The screen darkens slightly toward the bottom edge, as in the reference.

Look at the reference screenshots in `/tmp/claude-0/-home-user-trapiks-game/74a64fa3-bf2f-5a5c-88ae-f83b4d67c7ec/scratchpad/style-ref/` (especially 1 and 2) before starting.

## Elevated shadows

- **Containers:** `layers.ts` adds a `shadow` container before `outline` for each layer from 1 to 5.
  - Its filters are `[new BlurFilter({ strength, quality: 3 }), new AlphaFilter({ alpha: 0.13 })]`, imported from `pixi.js`.
  - `strength = clamp(1.5 × scale, 1, 10)` px, updated when the scale changes by more than 10 %.
  - Filtering the whole container means overlapping shadows do not darken twice.
- **Tile builder:** for elevated roads and elevated junction shapes (layer ≥ 1) in detail tiles, emit a `shadow` piece.
  - It holds the same strokes as the outline pass (width `W + 1.0`) and the junction polygons filled, all in black `#000000`.
  - The piece's `Graphics.position` is set to `(6 × layer, 8 × layer)` metres, rather than translating points.
- **Scope:** city tiles emit no shadows, and the shadow containers are hidden while the city band is active.
- **Overlay:** `hud/debug-overlay.ts` adds `data-shadow-pieces`, the number of shadow pieces in visible tiles.

## Signal pills

### Rust (`crates/sim-core/src/render/signal_pills.rs`)

- `pub fn signal_pills(network: &Network) -> SignalPills` covers every signal approach link, where `network.signal_state(link, 0).is_some()`, in link order.
- `(c, d) = centre_pose(link, length − setback)`. The pill sits at `c + perp_right(d)·(W/2 + 1.2)` with `angle = atan2(d.y, d.x)`.
- `SignalPills { link: Vec<u32>, x: Vec<f32>, y: Vec<f32>, angle: Vec<f32> }`.
- `pub fn signal_states(network: &Network, pills: &[u32], tick: u64, out: &mut Vec<u8>)` writes one code per pill: `0` green, `1` amber, `2` red.

### wasm

- `Engine.signalPills()` returns the geometry. The worker adds it to `ready`.
- `Engine.signalStates() -> Vec<u8>` works at the current tick.

### Worker

- After each wall tick, the worker calls `signalStates()` and compares the result with the previous array.
- When it changed, it posts `{ type: 'signals', states: Uint8Array }` (transferred).
- The first post happens right after `ready`.

### Main thread (`render/signal-pills.ts`)

- One `ParticleContainer` in `layers.overlay`, with `dynamicProperties: { position: false, rotation: false, color: true }`.
- One `Particle` per pill, using a white rounded-rect texture of 1.4 × 0.7 m at 20 px/m, `resolution: 4`, with a thin darker border.
- `tint` comes from the state: `#43B581` green, `#F2B33D` amber, `#E5484D` red.
- Visible only while `scale ≥ STREET_MIN_SCALE`.
- **Overlay:** `hud/debug-overlay.ts` adds `data-signal-pills` (the count) and `data-signal-updates` (how many `signals` messages have arrived).

## Vehicle sprite

`render/vehicle-texture.ts` draws a 4.4 × 1.9 m car at 10 px/m with `resolution: 4`:
- **Body:** a rounded rect (radius 0.45 m) filled white with a 0.12 m light grey `#CFCFCF` border. Under tint, the border reads as a darker shade of the body colour.
- **Windshield:** a dark `#1F2330` bar 0.35 m long, across 80 % of the width, centred at 70 % of the length toward the front.
- **Rear window:** a dark bar 0.25 m long at 22 % of the length.

The front is +x, matching heading 0.

## Bottom vignette

`hud/styles.css`: `#app::after` is a fixed, full-screen, `pointer-events: none` overlay with `background: linear-gradient(to bottom, transparent 65%, rgba(0,0,0,0.07))`.

## Tests

- **Rust:** `signal_pills` gives 4 pills on `four_way`. `signal_states` at ticks 0, 300, 330 and 350 matches M3's signal windows.
- **vitest:** `signal-pills.test.ts` covers the state-to-tint mapping and visibility by scale, with the pure parts factored out.
- **Playwright** (`e2e/depth.spec.ts`):
  1. **Skyway ramp,** at `/?map=synthetic&vph=0&cx=2300&cy=2300&z=5`:
     - Wait until the tiles are built.
     - `data-shadow-pieces > 0`.
     - Save the screenshot `depth-ramp.png`.
  2. **Signal junction,** at `/?map=synthetic&vph=0&cx=750&cy=750&z=12`:
     - `data-signal-pills` is at least 4.
     - Press `5` (max) and wait 3 s: `data-signal-updates` is at least 2.
     - Save the screenshot `depth-signals.png`.
  3. **Vehicles,** at `/?map=synthetic&vph=6000&cx=750&cy=900&z=10`:
     - Wait 5 s.
     - Save the screenshot `depth-vehicles.png`.
  4. No console errors in any test.

## Acceptance

- `bash scripts/check.sh` passes.
- Report the three screenshot paths. The orchestrator compares them with the reference.
- No comments. Do not modify `CLAUDE.md`, `docs/` or `.claude/`.
