import { cumulative, type Point } from '../polyline';

export interface Pose {
  x: number;
  y: number;
  tx: number;
  ty: number;
}

export interface RoadWalk {
  length: number;
  pose: (s: number) => Pose;
}

function poseOn(a: Point, b: Point, t: number): Pose {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy) || 1;
  return { x: a.x + dx * t, y: a.y + dy * t, tx: dx / len, ty: dy / len };
}

export function walkRoad(points: readonly Point[]): RoadWalk {
  const lengths = cumulative(points);
  const length = lengths[lengths.length - 1] ?? 0;
  const pose = (s: number): Pose => {
    const found = lengths.findIndex((c, i) => i > 0 && s <= c);
    const i = found < 0 ? lengths.length - 1 : found;
    const a = points[i - 1] ?? points[0] ?? { x: 0, y: 0 };
    const b = points[i] ?? a;
    const start = lengths[i - 1] ?? 0;
    const span = (lengths[i] ?? 0) - start;
    return poseOn(a, b, span > 0 ? (s - start) / span : 0);
  };
  return { length, pose };
}
