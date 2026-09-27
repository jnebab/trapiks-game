import { describe, expect, it } from 'vitest';
import { badgeDistances, endLevel, rampBadges } from './badge-placement';

describe('badgeDistances', () => {
  it('skips roads shorter than 60 m', () => {
    expect(badgeDistances(59)).toEqual([]);
  });

  it('puts one badge in the middle of a short elevated road', () => {
    expect(badgeDistances(60)).toEqual([30]);
    expect(badgeDistances(200)).toEqual([100]);
  });

  it('spaces badges about 150 m apart', () => {
    const distances = badgeDistances(620);
    expect(distances).toHaveLength(4);
    expect((distances[1] ?? 0) - (distances[0] ?? 0)).toBeCloseTo(155);
    expect(distances[0]).toBeCloseTo(77.5);
  });
});

describe('endLevel', () => {
  it('keeps the own layer when a neighbour continues it', () => {
    expect(endLevel(1, [0, 1])).toBe(1);
  });

  it('uses the nearest neighbour layer otherwise', () => {
    expect(endLevel(0, [1, 2])).toBe(1);
    expect(endLevel(1, [0, 2])).toBe(0);
    expect(endLevel(2, [])).toBe(2);
  });
});

describe('rampBadges', () => {
  const ramp = { length: 100, fromLevel: 0, toLevel: 1 };

  it('marks a climbing one-way ramp at its entry', () => {
    expect(rampBadges({ ...ramp, forward: true, backward: false })).toEqual([
      { distance: 12, climb: 'up' },
    ]);
  });

  it('marks a descending one-way ramp driven backwards', () => {
    expect(rampBadges({ ...ramp, forward: false, backward: true })).toEqual([
      { distance: 88, climb: 'down' },
    ]);
  });

  it('marks both entries of a two-way ramp', () => {
    expect(rampBadges({ ...ramp, forward: true, backward: true })).toHaveLength(2);
  });

  it('ignores level roads', () => {
    expect(rampBadges({ ...ramp, toLevel: 0, forward: true, backward: true })).toEqual([]);
  });
});
