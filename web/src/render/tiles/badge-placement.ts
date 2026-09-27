export const BADGE_SPACING = 150;
export const BADGE_MIN_LENGTH = 60;
const RAMP_INSET = 12;
const RAMP_INSET_SHARE = 0.25;

export type Climb = 'up' | 'down';

export interface RampBadge {
  distance: number;
  climb: Climb;
}

export function badgeDistances(length: number): number[] {
  if (length < BADGE_MIN_LENGTH) {
    return [];
  }
  const count = Math.max(1, Math.floor(length / BADGE_SPACING));
  const step = length / count;
  return Array.from({ length: count }, (_, i) => (i + 0.5) * step);
}

export function endLevel(own: number, neighbours: readonly number[]): number {
  if (neighbours.length === 0 || neighbours.includes(own)) {
    return own;
  }
  const byGap = [...neighbours].sort((a, b) => Math.abs(a - own) - Math.abs(b - own) || a - b);
  return byGap[0] ?? own;
}

export interface RampEnds {
  length: number;
  fromLevel: number;
  toLevel: number;
  forward: boolean;
  backward: boolean;
}

function entryBadge(ends: RampEnds, atStart: boolean): RampBadge {
  const inset = Math.min(RAMP_INSET, ends.length * RAMP_INSET_SHARE);
  const entry = atStart ? ends.fromLevel : ends.toLevel;
  const exit = atStart ? ends.toLevel : ends.fromLevel;
  return {
    distance: atStart ? inset : ends.length - inset,
    climb: exit > entry ? 'up' : 'down',
  };
}

export function rampBadges(ends: RampEnds): RampBadge[] {
  if (ends.fromLevel === ends.toLevel) {
    return [];
  }
  const badges: RampBadge[] = [];
  if (ends.forward) {
    badges.push(entryBadge(ends, true));
  }
  if (ends.backward) {
    badges.push(entryBadge(ends, false));
  }
  return badges;
}
