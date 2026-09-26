import { describe, expect, it } from 'vitest';
import { snapEndpoint, type SnapLookups } from './snap';

const ROAD = [
  { x: 0, y: 0 },
  { x: 100, y: 0 },
];

function lookups(node: number | undefined, road: number | undefined): SnapLookups {
  return {
    metresPerPixel: 0.5,
    pickNode: (_x, _y, radius) => (radius === 6 ? node : undefined),
    pickRoad: (_x, _y, tolerance) => (tolerance === 4 ? road : undefined),
    roadPoints: () => ROAD,
    roadEnds: () => [7, 8],
    nodePoint: (id) => ({ x: id, y: id }),
  };
}

describe('snapEndpoint', () => {
  it('prefers a node within 12 px', () => {
    expect(snapEndpoint(40, 2, lookups(3, 5))).toEqual({
      endpoint: { Node: { node: 3 } },
      point: { x: 3, y: 3 },
    });
  });

  it('projects onto a road within 8 px', () => {
    expect(snapEndpoint(40, 2, lookups(undefined, 5))).toEqual({
      endpoint: { OnRoad: { road: 5, at_m: 40 } },
      point: { x: 40, y: 0 },
    });
  });

  it('snaps near either end to the end node', () => {
    expect(snapEndpoint(6, 1, lookups(undefined, 5))?.endpoint).toEqual({ Node: { node: 7 } });
    expect(snapEndpoint(95, 1, lookups(undefined, 5))?.endpoint).toEqual({ Node: { node: 8 } });
  });

  it('gives null away from the network', () => {
    expect(snapEndpoint(40, 30, lookups(undefined, undefined))).toBeNull();
  });
});
