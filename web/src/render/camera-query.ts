import { MAX_SCALE, MIN_SCALE, type Bounds, type Camera } from './camera';

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function readNumber(params: URLSearchParams, name: string): number | null {
  const raw = params.get(name);
  const value = raw === null ? NaN : Number(raw);
  return Number.isFinite(value) ? value : null;
}

export interface ViewSize {
  w: number;
  h: number;
}

export function cameraFromQuery(
  search: string,
  bounds: Bounds,
  view: ViewSize,
  fallback: Camera,
): Camera {
  const params = new URLSearchParams(search);
  const [minX, minY, maxX, maxY] = bounds;
  const [fallbackX, fallbackY] = [
    (view.w / 2 - fallback.x) / fallback.scale,
    (view.h / 2 - fallback.y) / fallback.scale,
  ];
  const cx = clamp(readNumber(params, 'cx') ?? fallbackX, minX, maxX);
  const cy = clamp(readNumber(params, 'cy') ?? fallbackY, minY, maxY);
  const scale = clamp(readNumber(params, 'z') ?? fallback.scale, MIN_SCALE, MAX_SCALE);
  return { x: view.w / 2 - cx * scale, y: view.h / 2 - cy * scale, scale };
}
