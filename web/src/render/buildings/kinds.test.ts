import { describe, expect, it } from 'vitest';
import { buildingKind } from './kinds';

const always = (value: number) => () => value;

describe('buildingKind', () => {
  it('puts houses on residential streets', () => {
    expect(buildingKind('Residential', always(0))).toBe('house');
    expect(buildingKind('LivingStreet', always(0))).toBe('house');
  });

  it('puts commercial blocks on main roads', () => {
    for (const roadClass of ['Primary', 'Secondary', 'Trunk'] as const) {
      expect(buildingKind(roadClass, always(0))).toBe('commercial');
    }
  });

  it('mixes warehouses into minor roads', () => {
    expect(buildingKind('Tertiary', always(0.1))).toBe('warehouse');
    expect(buildingKind('Unclassified', always(0.5))).toBe('house');
    expect(buildingKind('Road', always(0.29))).toBe('warehouse');
  });

  it('skips every other class', () => {
    expect(buildingKind('Motorway', always(0))).toBeUndefined();
    expect(buildingKind('Service', always(0))).toBeUndefined();
    expect(buildingKind(undefined, always(0))).toBeUndefined();
  });
});
