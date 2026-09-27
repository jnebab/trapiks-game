export interface Obb {
  cx: number;
  cy: number;
  ux: number;
  uy: number;
  halfW: number;
  halfD: number;
}

export type Corner = readonly [number, number];

export function corners(box: Obb): Corner[] {
  const wx = box.ux * box.halfW;
  const wy = box.uy * box.halfW;
  const dx = -box.uy * box.halfD;
  const dy = box.ux * box.halfD;
  return [
    [box.cx - wx - dx, box.cy - wy - dy],
    [box.cx + wx - dx, box.cy + wy - dy],
    [box.cx + wx + dx, box.cy + wy + dy],
    [box.cx - wx + dx, box.cy - wy + dy],
  ];
}

export function flat(points: readonly Corner[]): number[] {
  return points.flatMap(([x, y]) => [x, y]);
}

function axes(box: Obb): Corner[] {
  return [
    [box.ux, box.uy],
    [-box.uy, box.ux],
  ];
}

function radius(box: Obb, [ax, ay]: Corner): number {
  const along = Math.abs(box.ux * ax + box.uy * ay);
  const across = Math.abs(-box.uy * ax + box.ux * ay);
  return along * box.halfW + across * box.halfD;
}

function separated(a: Obb, b: Obb, axis: Corner): boolean {
  const gap = Math.abs((b.cx - a.cx) * axis[0] + (b.cy - a.cy) * axis[1]);
  return gap >= radius(a, axis) + radius(b, axis);
}

export function overlaps(a: Obb, b: Obb): boolean {
  return ![...axes(a), ...axes(b)].some((axis) => separated(a, b, axis));
}

export function distanceTo(box: Obb, x: number, y: number): number {
  const rx = x - box.cx;
  const ry = y - box.cy;
  const along = Math.max(Math.abs(rx * box.ux + ry * box.uy) - box.halfW, 0);
  const across = Math.max(Math.abs(-rx * box.uy + ry * box.ux) - box.halfD, 0);
  return Math.hypot(along, across);
}

export function extent(box: Obb): number {
  return Math.hypot(box.halfW, box.halfD);
}
