import type { RoadArrays } from '../../sim/protocol';
import type { Rect } from '../rect';

function pointBounds(roads: RoadArrays, road: number): Rect {
  const start = roads.pointStart[road] ?? 0;
  const end = roads.pointStart[road + 1] ?? start;
  const r = { minX: Infinity, minY: Infinity, maxX: -Infinity, maxY: -Infinity };
  for (let i = start; i < end; i += 1) {
    const x = roads.x[i] ?? 0;
    const y = roads.y[i] ?? 0;
    r.minX = Math.min(r.minX, x);
    r.minY = Math.min(r.minY, y);
    r.maxX = Math.max(r.maxX, x);
    r.maxY = Math.max(r.maxY, y);
  }
  return r;
}

export function roadBounds(roads: RoadArrays): Float32Array {
  const count = roads.layer.length;
  const out = new Float32Array(count * 4);
  for (let road = 0; road < count; road += 1) {
    const r = pointBounds(roads, road);
    out.set([r.minX, r.minY, r.maxX, r.maxY], road * 4);
  }
  return out;
}

export function boundsOf(bounds: Float32Array, road: number): Rect {
  const i = road * 4;
  return {
    minX: bounds[i] ?? 0,
    minY: bounds[i + 1] ?? 0,
    maxX: bounds[i + 2] ?? 0,
    maxY: bounds[i + 3] ?? 0,
  };
}
