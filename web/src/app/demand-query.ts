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
