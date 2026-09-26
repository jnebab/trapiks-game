import type { Rect } from '../rect';
import type { RoadStore } from '../road-store';

export function roadBox(roads: RoadStore, road: number): Rect {
  const [start, end] = roads.pointRange(road);
  const r = { minX: Infinity, minY: Infinity, maxX: -Infinity, maxY: -Infinity };
  for (let i = start; i < end; i += 1) {
    const x = roads.xs[i] ?? 0;
    const y = roads.ys[i] ?? 0;
    r.minX = Math.min(r.minX, x);
    r.minY = Math.min(r.minY, y);
    r.maxX = Math.max(r.maxX, x);
    r.maxY = Math.max(r.maxY, y);
  }
  return r;
}
