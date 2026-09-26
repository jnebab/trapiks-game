import { describe, expect, it } from 'vitest';
import { expand, intersects } from './rect';

const unit = { minX: 0, minY: 0, maxX: 1, maxY: 1 };

describe('rect', () => {
  it('intersects overlapping and touching rects', () => {
    expect(intersects(unit, { minX: 0.5, minY: 0.5, maxX: 2, maxY: 2 })).toBe(true);
    expect(intersects(unit, { minX: 1, minY: 0, maxX: 2, maxY: 1 })).toBe(true);
  });

  it('rejects separated rects', () => {
    expect(intersects(unit, { minX: 1.1, minY: 0, maxX: 2, maxY: 1 })).toBe(false);
    expect(intersects(unit, { minX: 0, minY: -2, maxX: 1, maxY: -0.5 })).toBe(false);
  });

  it('expands on every side', () => {
    expect(expand(unit, 2)).toEqual({ minX: -2, minY: -2, maxX: 3, maxY: 3 });
  });
});
