import { Graphics, type Renderer, type Texture } from 'pixi.js';
import { WINDSHIELD } from './palette';

const PX_PER_M = 10;
const LENGTH_PX = 4.4 * PX_PER_M;
const WIDTH_PX = 1.9 * PX_PER_M;
const CORNER_PX = 0.45 * PX_PER_M;
const BORDER_PX = 0.12 * PX_PER_M;
const BORDER_COLOR = 0xcfcfcf;
const GLASS_SPAN = 0.8;
const FRONT_GLASS = { at: 0.7, length: 0.35 * PX_PER_M } as const;
const REAR_GLASS = { at: 0.22, length: 0.25 * PX_PER_M } as const;
const RESOLUTION = 4;

function glassBar(g: Graphics, bar: { at: number; length: number }): Graphics {
  const span = WIDTH_PX * GLASS_SPAN;
  return g
    .rect(LENGTH_PX * bar.at - bar.length / 2, (WIDTH_PX - span) / 2, bar.length, span)
    .fill(WINDSHIELD);
}

export function createVehicleTexture(renderer: Renderer): Texture {
  const inset = BORDER_PX / 2;
  const body = new Graphics()
    .roundRect(inset, inset, LENGTH_PX - BORDER_PX, WIDTH_PX - BORDER_PX, CORNER_PX)
    .fill(0xffffff)
    .stroke({ width: BORDER_PX, color: BORDER_COLOR });
  const target = glassBar(glassBar(body, FRONT_GLASS), REAR_GLASS);
  const texture = renderer.generateTexture({ target, resolution: RESOLUTION, antialias: true });
  target.destroy();
  return texture;
}
