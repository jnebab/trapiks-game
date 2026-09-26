import { describe, expect, it } from 'vitest';
import { VEHICLE_COLORS } from './palette';
import { vehicleKindOf, vehicleTintOf } from './vehicle-style';

describe('vehicle style decode', () => {
  it('decodes every kind and colour', () => {
    VEHICLE_COLORS.forEach((color, index) => {
      for (const kind of [0, 1, 2]) {
        const style = kind * 16 + index;
        expect(vehicleKindOf(style)).toBe(kind);
        expect(vehicleTintOf(style)).toBe(kind === 0 ? color : 0xffffff);
      }
    });
  });

  it('clamps unknown kinds to the last frame', () => {
    expect(vehicleKindOf(3 * 16)).toBe(2);
  });
});
