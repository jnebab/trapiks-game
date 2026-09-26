import { describe, expect, it } from 'vitest';
import fixture from './curve-fixture.json';
import { ADD_ROAD_SEGMENTS, roadGeometry } from './road-preview';

function point([x, y]: number[]): { x: number; y: number } {
  return { x: x ?? 0, y: y ?? 0 };
}

describe('roadGeometry', () => {
  it('matches the Rust curve samples', () => {
    const points = roadGeometry(point(fixture.from), point(fixture.to), point(fixture.via));
    expect(points).toHaveLength(ADD_ROAD_SEGMENTS + 1);
    expect(points).toEqual(fixture.points.map(point));
  });

  it('is a single segment without a control point', () => {
    const from = { x: 0, y: 0 };
    const to = { x: 10, y: 5 };
    expect(roadGeometry(from, to, undefined)).toEqual([from, to]);
  });
});
