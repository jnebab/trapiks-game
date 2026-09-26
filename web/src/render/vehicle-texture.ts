import { Graphics, type Renderer, type Texture } from 'pixi.js';
import { WINDSHIELD } from './palette';

const LENGTH_PX = 44;
const WIDTH_PX = 19;
const CORNER_PX = 5;
const WINDSHIELD_AT = 0.7;
const WINDSHIELD_PX = 5;
const WINDSHIELD_INSET_PX = 2;
const RESOLUTION = 4;

export function createVehicleTexture(renderer: Renderer): Texture {
  const target = new Graphics()
    .roundRect(0, 0, LENGTH_PX, WIDTH_PX, CORNER_PX)
    .fill(0xffffff)
    .rect(
      LENGTH_PX * WINDSHIELD_AT - WINDSHIELD_PX / 2,
      WINDSHIELD_INSET_PX,
      WINDSHIELD_PX,
      WIDTH_PX - 2 * WINDSHIELD_INSET_PX,
    )
    .fill(WINDSHIELD);
  const texture = renderer.generateTexture({ target, resolution: RESOLUTION, antialias: true });
  target.destroy();
  return texture;
}
