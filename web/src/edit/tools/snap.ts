import type { Endpoint } from '../../generated/Endpoint';
import type { Point } from '../../render/polyline';

export const NODE_SNAP_PX = 12;
export const ROAD_SNAP_PX = 8;
export const END_MARGIN_M = 10;

export interface SnapLookups {
  metresPerPixel: number;
  pickNode: (x: number, y: number, radius: number) => number | undefined;
  pickRoad: (x: number, y: number, tolerance: number) => number | undefined;
  roadPoints: (road: number) => readonly Point[];
  roadEnds: (road: number) => readonly [number, number];
  nodePoint: (node: number) => Point;
}

export interface Snapped {
  endpoint: Endpoint;
  point: Point;
}

export interface Projection {
  atM: number;
  length: number;
  point: Point;
}

interface Nearest {
  distance: number;
  atM: number;
  point: Point;
}

function projectOnSegment(a: Point, b: Point, x: number, y: number): { t: number; point: Point } {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const lengthSq = dx * dx + dy * dy;
  const raw = lengthSq > 0 ? ((x - a.x) * dx + (y - a.y) * dy) / lengthSq : 0;
  const t = Math.min(1, Math.max(0, raw));
  return { t, point: { x: a.x + dx * t, y: a.y + dy * t } };
}

export function projectOnPolyline(points: readonly Point[], x: number, y: number): Projection {
  let travelled = 0;
  let best: Nearest = { distance: Infinity, atM: 0, point: points[0] ?? { x, y } };
  for (let i = 0; i + 1 < points.length; i += 1) {
    const a = points[i] ?? { x, y };
    const b = points[i + 1] ?? a;
    const span = Math.hypot(b.x - a.x, b.y - a.y);
    const { t, point } = projectOnSegment(a, b, x, y);
    const distance = Math.hypot(point.x - x, point.y - y);
    if (distance < best.distance) {
      best = { distance, atM: travelled + span * t, point };
    }
    travelled += span;
  }
  return { atM: best.atM, length: travelled, point: best.point };
}

function nodeSnap(node: number, lookups: SnapLookups): Snapped {
  return { endpoint: { Node: { node } }, point: lookups.nodePoint(node) };
}

function roadSnap(road: number, x: number, y: number, lookups: SnapLookups): Snapped {
  const projection = projectOnPolyline(lookups.roadPoints(road), x, y);
  const [from, to] = lookups.roadEnds(road);
  if (projection.atM < END_MARGIN_M) {
    return nodeSnap(from, lookups);
  }
  if (projection.atM > projection.length - END_MARGIN_M) {
    return nodeSnap(to, lookups);
  }
  return { endpoint: { OnRoad: { road, at_m: projection.atM } }, point: projection.point };
}

export function snapEndpoint(x: number, y: number, lookups: SnapLookups): Snapped | null {
  const node = lookups.pickNode(x, y, NODE_SNAP_PX * lookups.metresPerPixel);
  if (node !== undefined) {
    return nodeSnap(node, lookups);
  }
  const road = lookups.pickRoad(x, y, ROAD_SNAP_PX * lookups.metresPerPixel);
  return road === undefined ? null : roadSnap(road, x, y, lookups);
}
