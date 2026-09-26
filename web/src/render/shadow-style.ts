export const SHADOW_OFFSET = { x: 6, y: 8 } as const;

const BLUR_PER_SCALE = 2.5;
const BLUR_MIN = 1;
const BLUR_MAX = 16;
const RESCALE_THRESHOLD = 0.1;

export function shadowBlurStrength(scale: number): number {
  return Math.min(Math.max(BLUR_PER_SCALE * scale, BLUR_MIN), BLUR_MAX);
}

export function shadowNeedsUpdate(applied: number, scale: number): boolean {
  return Math.abs(scale - applied) > RESCALE_THRESHOLD * applied;
}
