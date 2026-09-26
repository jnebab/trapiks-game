import { describe, expect, it } from 'vitest';
import { arrowsAlong, cumulative, dashes, offset, slice, type Point } from './polyline';

const line: Point[] = [
  { x: 0, y: 0 },
  { x: 30, y: 0 },
];

const corner: Point[] = [
  { x: 0, y: 0 },
  { x: 10, y: 0 },
  { x: 10, y: 10 },
];

function close(a: Point | undefined, b: Point): void {
  expect(a?.x).toBeCloseTo(b.x, 6);
  expect(a?.y).toBeCloseTo(b.y, 6);
}

describe('polyline offsets', () => {
  it('measures cumulative length', () => {
    expect(cumulative(corner)).toEqual([0, 10, 20]);
  });

  it('offsets a straight line to the right in y-down', () => {
    const shifted = offset(line, 2);
    close(shifted[0], { x: 0, y: 2 });
    close(shifted[1], { x: 30, y: 2 });
  });

  it('offsets a right angle with a mitred corner', () => {
    const shifted = offset(corner, -1);
    close(shifted[1], { x: 11, y: -1 });
  });

  it('caps the miter at twice the offset', () => {
    const hairpin: Point[] = [
      { x: 0, y: 0 },
      { x: 10, y: 0 },
      { x: 0, y: 0.1 },
    ];
    const shifted = offset(hairpin, 1);
    const p = shifted[1] ?? { x: 0, y: 0 };
    expect(Math.hypot(p.x - 10, p.y)).toBeCloseTo(2, 6);
  });
});

describe('polyline spans', () => {
  it('cuts 5 dashes from a 30 m line at 3/3', () => {
    const pieces = dashes(line, 3, 3);
    expect(pieces).toHaveLength(5);
    close(pieces[4]?.[0], { x: 24, y: 0 });
    close(pieces[4]?.[1], { x: 27, y: 0 });
  });

  it('slices at and beyond the bounds', () => {
    expect(slice(corner, 0, 20)).toEqual(corner);
    expect(slice(corner, -5, 50)).toEqual(corner);
    expect(slice(corner, 5, 15)).toEqual([
      { x: 5, y: 0 },
      { x: 10, y: 0 },
      { x: 10, y: 5 },
    ]);
    expect(slice(corner, 12, 12)).toEqual([]);
  });

  it('places arrows with the segment angle', () => {
    const arrows = arrowsAlong(corner, 10, 5);
    expect(arrows).toHaveLength(2);
    expect(arrows[0]?.angle).toBeCloseTo(0, 6);
    expect(arrows[1]?.angle).toBeCloseTo(Math.PI / 2, 6);
    close(arrows[1], { x: 10, y: 5 });
  });
});
