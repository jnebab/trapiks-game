import { Graphics } from 'pixi.js';
import type { AreaKind } from '../generated/AreaKind';
import type { AreaArrays } from '../sim/protocol';
import { palette } from './palette';

const areaColors: Record<AreaKind, number> = {
  Water: palette.water,
  Park: palette.park,
};

function ringPoints(areas: AreaArrays, ring: number): number[] {
  const start = areas.ringStart[ring] ?? 0;
  const end = areas.ringStart[ring + 1] ?? start;
  const points: number[] = [];
  for (let i = start; i < end; i += 1) {
    points.push(areas.x[i] ?? 0, areas.y[i] ?? 0);
  }
  return points;
}

export function drawAreas(areas: AreaArrays, kindNames: readonly AreaKind[]): Graphics {
  const g = new Graphics();
  for (let ring = 0; ring < areas.kindCode.length; ring += 1) {
    const kind = kindNames[areas.kindCode[ring] ?? 0];
    if (kind === undefined) {
      continue;
    }
    g.poly(ringPoints(areas, ring), true).fill(areaColors[kind]);
  }
  return g;
}
