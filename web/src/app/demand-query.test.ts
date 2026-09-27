import { describe, expect, it } from 'vitest';
import { regionFromQuery } from './demand-query';

describe('regionFromQuery', () => {
  it('reads a region centre and radius', () => {
    expect(regionFromQuery('?region=9000,9000,2000')).toEqual({
      Region: { center_x: 9000, center_y: 9000, radius: 2000 },
    });
  });

  it('falls back to the whole city', () => {
    expect(regionFromQuery('?map=grid120')).toBe('City');
    expect(regionFromQuery('?region=1,2')).toBe('City');
    expect(regionFromQuery('?region=1,2,-5')).toBe('City');
  });
});
