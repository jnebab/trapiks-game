import type { Graphics } from 'pixi.js';
import type { RoadArrays } from '../../sim/protocol';

export function tracePolyline(g: Graphics, roads: RoadArrays, road: number): void {
  const start = roads.pointStart[road] ?? 0;
  const end = roads.pointStart[road + 1] ?? start;
  g.moveTo(roads.x[start] ?? 0, roads.y[start] ?? 0);
  for (let i = start + 1; i < end; i += 1) {
    g.lineTo(roads.x[i] ?? 0, roads.y[i] ?? 0);
  }
}
