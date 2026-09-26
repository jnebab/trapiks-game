import type { Point } from '../../render/polyline';

export const ADD_ROAD_SEGMENTS = 16;

function bezier(from: Point, via: Point, to: Point, t: number): Point {
  const u = 1 - t;
  return {
    x: from.x * (u * u) + via.x * (2 * u * t) + to.x * (t * t),
    y: from.y * (u * u) + via.y * (2 * u * t) + to.y * (t * t),
  };
}

export function roadGeometry(from: Point, to: Point, via: Point | undefined): Point[] {
  if (via === undefined) {
    return [from, to];
  }
  const points: Point[] = [];
  for (let k = 0; k <= ADD_ROAD_SEGMENTS; k += 1) {
    points.push(bezier(from, via, to, k / ADD_ROAD_SEGMENTS));
  }
  points[0] = from;
  points[ADD_ROAD_SEGMENTS] = to;
  return points;
}
