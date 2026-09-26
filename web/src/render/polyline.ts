export interface Point {
  x: number;
  y: number;
}

export interface Arrow {
  x: number;
  y: number;
  angle: number;
}

const MITER_LIMIT = 2;
const EPSILON = 1e-9;

export function cumulative(points: readonly Point[]): number[] {
  const out: number[] = [];
  let total = 0;
  points.forEach((p, i) => {
    const prev = points[i - 1];
    total += prev === undefined ? 0 : Math.hypot(p.x - prev.x, p.y - prev.y);
    out.push(total);
  });
  return out;
}

function lerp(a: Point, b: Point, t: number): Point {
  return { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t };
}

function segmentEnd(lengths: readonly number[], s: number): number {
  const found = lengths.findIndex((c, i) => i > 0 && s <= c);
  return found < 0 ? lengths.length - 1 : found;
}

function pointAt(points: readonly Point[], lengths: readonly number[], s: number): Point {
  const i = segmentEnd(lengths, s);
  const a = points[i - 1];
  const b = points[i];
  if (a === undefined || b === undefined) {
    return points[points.length - 1] ?? { x: 0, y: 0 };
  }
  const start = lengths[i - 1] ?? 0;
  const span = (lengths[i] ?? 0) - start;
  return lerp(a, b, span > EPSILON ? (s - start) / span : 0);
}

export function slice(points: readonly Point[], from: number, to: number): Point[] {
  const lengths = cumulative(points);
  const total = lengths[lengths.length - 1] ?? 0;
  const lo = Math.max(from, 0);
  const hi = Math.min(to, total);
  if (hi - lo <= EPSILON) {
    return [];
  }
  const inner = points.filter((_, i) => {
    const s = lengths[i] ?? 0;
    return s > lo && s < hi;
  });
  return [pointAt(points, lengths, lo), ...inner, pointAt(points, lengths, hi)];
}

function unitNormal(a: Point, b: Point): Point {
  const length = Math.hypot(b.x - a.x, b.y - a.y);
  if (length < EPSILON) {
    return { x: 0, y: 0 };
  }
  return { x: -(b.y - a.y) / length, y: (b.x - a.x) / length };
}

function vertexNormal(prev: Point, next: Point): Point {
  const sum = { x: prev.x + next.x, y: prev.y + next.y };
  const length = Math.hypot(sum.x, sum.y);
  if (length < EPSILON) {
    return prev;
  }
  const m = { x: sum.x / length, y: sum.y / length };
  const cos = m.x * prev.x + m.y * prev.y;
  const scale = Math.min(1 / Math.max(cos, EPSILON), MITER_LIMIT);
  return { x: m.x * scale, y: m.y * scale };
}

function normalAt(points: readonly Point[], i: number): Point {
  const prev = points[i - 1];
  const here = points[i];
  const next = points[i + 1];
  if (here === undefined) {
    return { x: 0, y: 0 };
  }
  const before = prev === undefined ? undefined : unitNormal(prev, here);
  const after = next === undefined ? undefined : unitNormal(here, next);
  if (before === undefined || after === undefined) {
    return before ?? after ?? { x: 0, y: 0 };
  }
  return vertexNormal(before, after);
}

export function offset(points: readonly Point[], d: number): Point[] {
  return points.map((p, i) => {
    const n = normalAt(points, i);
    return { x: p.x + n.x * d, y: p.y + n.y * d };
  });
}

export function dashes(points: readonly Point[], dash: number, gap: number): Point[][] {
  const lengths = cumulative(points);
  const total = lengths[lengths.length - 1] ?? 0;
  const out: Point[][] = [];
  for (let s = 0; s < total; s += dash + gap) {
    const piece = slice(points, s, Math.min(s + dash, total));
    if (piece.length > 1) {
      out.push(piece);
    }
  }
  return out;
}

function angleAt(points: readonly Point[], lengths: readonly number[], s: number): number {
  const i = segmentEnd(lengths, s);
  const a = points[i - 1] ?? { x: 0, y: 0 };
  const b = points[i] ?? a;
  return Math.atan2(b.y - a.y, b.x - a.x);
}

export function arrowsAlong(points: readonly Point[], spacing: number, start: number): Arrow[] {
  const lengths = cumulative(points);
  const total = lengths[lengths.length - 1] ?? 0;
  const out: Arrow[] = [];
  for (let s = start; s <= total && points.length > 1; s += spacing) {
    const p = pointAt(points, lengths, s);
    out.push({ x: p.x, y: p.y, angle: angleAt(points, lengths, s) });
  }
  return out;
}
