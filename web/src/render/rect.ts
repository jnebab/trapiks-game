export interface Rect {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export function intersects(a: Rect, b: Rect): boolean {
  return a.minX <= b.maxX && b.minX <= a.maxX && a.minY <= b.maxY && b.minY <= a.maxY;
}

export function expand(r: Rect, margin: number): Rect {
  return {
    minX: r.minX - margin,
    minY: r.minY - margin,
    maxX: r.maxX + margin,
    maxY: r.maxY + margin,
  };
}
