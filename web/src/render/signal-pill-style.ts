import { STREET_MIN_SCALE } from './style';

export const PILL_TINTS: readonly number[] = [0x43b581, 0xf2b33d, 0xe5484d];
const RED = 0xe5484d;

export function pillTint(state: number): number {
  return PILL_TINTS[state] ?? RED;
}

export function pillsVisible(scale: number): boolean {
  return scale >= STREET_MIN_SCALE;
}
