export interface Camera {
  x: number;
  y: number;
  scale: number;
}

export type Bounds = readonly [number, number, number, number];

export const MIN_SCALE = 0.02;
export const MAX_SCALE = 40;

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function worldToScreen(c: Camera, wx: number, wy: number): [number, number] {
  return [wx * c.scale + c.x, wy * c.scale + c.y];
}

export function screenToWorld(c: Camera, sx: number, sy: number): [number, number] {
  return [(sx - c.x) / c.scale, (sy - c.y) / c.scale];
}

export function pan(c: Camera, dx: number, dy: number): Camera {
  return { x: c.x + dx, y: c.y + dy, scale: c.scale };
}

export interface ZoomLimits {
  min: number;
  max: number;
}

export type ScreenPoint = readonly [number, number];

export function zoomAt(c: Camera, anchor: ScreenPoint, factor: number, limits: ZoomLimits): Camera {
  const [sx, sy] = anchor;
  const scale = clamp(c.scale * factor, limits.min, limits.max);
  const ratio = scale / c.scale;
  return { x: sx - (sx - c.x) * ratio, y: sy - (sy - c.y) * ratio, scale };
}

export function fitBounds(bounds: Bounds, viewW: number, viewH: number, margin: number): Camera {
  const [minX, minY, maxX, maxY] = bounds;
  const w = maxX - minX;
  const h = maxY - minY;
  const fit = Math.min((viewW - 2 * margin) / w, (viewH - 2 * margin) / h);
  const scale = clamp(fit, MIN_SCALE, MAX_SCALE);
  const cx = (minX + maxX) / 2;
  const cy = (minY + maxY) / 2;
  return { x: viewW / 2 - cx * scale, y: viewH / 2 - cy * scale, scale };
}
