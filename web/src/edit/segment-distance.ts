export function segmentDistance(
  px: number,
  py: number,
  a: readonly [number, number],
  b: readonly [number, number],
): number {
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const lengthSq = dx * dx + dy * dy;
  const t = lengthSq === 0 ? 0 : ((px - a[0]) * dx + (py - a[1]) * dy) / lengthSq;
  const clamped = Math.min(Math.max(t, 0), 1);
  return Math.hypot(px - (a[0] + dx * clamped), py - (a[1] + dy * clamped));
}
