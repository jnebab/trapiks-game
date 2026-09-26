import { describe, expect, it } from 'vitest';
import { laneMarkings } from './markings';

describe('laneMarkings', () => {
  it('centres the line of a 2+2 road with one divider per side', () => {
    const m = laneMarkings(2, 2);
    expect(m.centerLine).toBeCloseTo(0, 9);
    expect(m.dividers).toHaveLength(2);
    expect(m.dividers[0]).toBeCloseTo(3.2, 9);
    expect(m.dividers[1]).toBeCloseTo(-3.2, 9);
  });

  it('shifts the centre line of a 3+1 road', () => {
    const m = laneMarkings(3, 1);
    expect(m.centerLine).toBeCloseTo(-3.2, 9);
    expect(m.dividers).toHaveLength(2);
  });

  it('has no centre line on one-way roads', () => {
    const m = laneMarkings(2, 0);
    expect(m.centerLine).toBeUndefined();
    expect(m.dividers).toEqual([0]);
  });
});
