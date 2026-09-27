import type { SimMode } from '../generated/SimMode';

export const DEFAULT_VPH = 3000;
const MAX_VPH = 200_000;

export function demandFromQuery(search: string): number | undefined {
  const raw = new URLSearchParams(search).get('vph');
  const value = raw === null ? NaN : Number(raw);
  if (!Number.isFinite(value)) {
    return undefined;
  }
  return Math.min(MAX_VPH, Math.max(0, value));
}

export function regionFromQuery(search: string): SimMode {
  const raw = new URLSearchParams(search).get('region');
  const parts = (raw ?? '').split(',').map(Number);
  const [x, y, radius] = parts;
  const valid = parts.length === 3 && parts.every(Number.isFinite);
  if (!valid || x === undefined || y === undefined || radius === undefined || radius <= 0) {
    return 'City';
  }
  return { Region: { center_x: x, center_y: y, radius } };
}
