import type { Graphics } from 'pixi.js';
import type { RoadStore } from '../road-store';

export function tracePolyline(g: Graphics, roads: RoadStore, road: number): void {
  const [start, end] = roads.pointRange(road);
  g.moveTo(roads.xs[start] ?? 0, roads.ys[start] ?? 0);
  for (let i = start + 1; i < end; i += 1) {
    g.lineTo(roads.xs[i] ?? 0, roads.ys[i] ?? 0);
  }
}
