import type { Graphics } from 'pixi.js';
import type { DetailStore } from '../detail-store';
import { cumulative, slice, type Point } from '../polyline';
import type { RoadStore } from '../road-store';

export function tracePolyline(g: Graphics, roads: RoadStore, road: number): void {
  const [start, end] = roads.pointRange(road);
  g.moveTo(roads.xs[start] ?? 0, roads.ys[start] ?? 0);
  for (let i = start + 1; i < end; i += 1) {
    g.lineTo(roads.xs[i] ?? 0, roads.ys[i] ?? 0);
  }
}

function tracePoints(g: Graphics, points: readonly Point[]): void {
  points.forEach((p, i) => {
    if (i === 0) {
      g.moveTo(p.x, p.y);
    } else {
      g.lineTo(p.x, p.y);
    }
  });
}

function trimsAt(roads: RoadStore, road: number, node: number): boolean {
  if (roads.isRingNode(node) && !roads.isRoundabout(road)) {
    return true;
  }
  return roads.layer(road) > 0 && roads.isMixedNode(node);
}

export function isTrimmed(roads: RoadStore, road: number): boolean {
  return trimsAt(roads, road, roads.from(road)) || trimsAt(roads, road, roads.to(road));
}

function ringTrim(roads: RoadStore, detail: DetailStore, road: number): [number, number] {
  const from = trimsAt(roads, road, roads.from(road)) ? detail.setback(road, 0) : 0;
  const to = trimsAt(roads, road, roads.to(road)) ? detail.setback(road, 1) : 0;
  return [from, to];
}

export function traceRoad(
  g: Graphics,
  roads: RoadStore,
  road: number,
  detail: DetailStore | undefined,
): void {
  const [from, to] = detail === undefined ? [0, 0] : ringTrim(roads, detail, road);
  if (from === 0 && to === 0) {
    tracePolyline(g, roads, road);
    return;
  }
  const points = roads.pointsOf(road);
  const total = cumulative(points).at(-1) ?? 0;
  tracePoints(g, slice(points, from, total - to));
}
