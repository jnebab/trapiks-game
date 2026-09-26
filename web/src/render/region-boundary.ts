import { Graphics } from 'pixi.js';
import type { Region } from '../sim/protocol';

const COLOR = 0x8e8e8e;
const WIDTH_PX = 2;
const DASH_PX = 10;
const GAP_PX = 7;
const MAX_DASHES = 1440;
const REDRAW_LOG_STEP = 0.1;

export interface RegionBoundary {
  graphics: Graphics;
  setScale: (scale: number) => void;
}

function dashCount(region: Region, scale: number): number {
  const circumferencePx = 2 * Math.PI * region.radius * scale;
  return Math.max(8, Math.min(MAX_DASHES, Math.floor(circumferencePx / (DASH_PX + GAP_PX))));
}

function draw(g: Graphics, region: Region, scale: number): void {
  const dashes = dashCount(region, scale);
  const step = (2 * Math.PI) / dashes;
  const sweep = step * (DASH_PX / (DASH_PX + GAP_PX));
  g.clear();
  for (let i = 0; i < dashes; i += 1) {
    const start = i * step;
    g.moveTo(
      region.x + region.radius * Math.cos(start),
      region.y + region.radius * Math.sin(start),
    );
    g.arc(region.x, region.y, region.radius, start, start + sweep);
  }
  g.stroke({ width: WIDTH_PX / scale, color: COLOR });
}

export function createRegionBoundary(region: Region): RegionBoundary {
  const graphics = new Graphics();
  let drawnScale = 0;
  return {
    graphics,
    setScale: (scale) => {
      if (drawnScale > 0 && Math.abs(Math.log(scale / drawnScale)) < REDRAW_LOG_STEP) {
        return;
      }
      drawnScale = scale;
      draw(graphics, region, scale);
    },
  };
}
