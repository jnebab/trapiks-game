import { describe, expect, it } from 'vitest';
import type { RoadArrays } from '../../sim/protocol';
import { roadBounds } from './road-bounds';
import { buildTileIndex } from './tile-index';

const PRIMARY = 4;
const RESIDENTIAL = 12;
const RANKS = [14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 3, 2, 1];

function makeRoads(): RoadArrays {
  const lines = [
    [10, 10, 100, 10],
    [1000, 50, 1100, 50],
    [1100, 100, 1200, 100],
    [-50, -50, -10, -10],
    [600, 600, 700, 700],
  ];
  return {
    pointStart: Uint32Array.from([0, 2, 4, 6, 8, 10]),
    x: Float32Array.from(lines.flatMap(([x0, , x1]) => [x0 ?? 0, x1 ?? 0])),
    y: Float32Array.from(lines.flatMap(([, y0, , y1]) => [y0 ?? 0, y1 ?? 0])),
    classCode: Uint8Array.from([PRIMARY, PRIMARY, RESIDENTIAL, PRIMARY, PRIMARY]),
    lanesForward: Uint8Array.from([1, 1, 1, 1, 1]),
    lanesBackward: Uint8Array.from([1, 1, 1, 1, 1]),
    layer: Int8Array.from([0, 0, 0, 0, 0]),
    name: Uint32Array.from([0, 0, 0, 0, 0]),
  };
}

const roads = makeRoads();
const bounds = roadBounds(roads);
const detail = { name: 'detail', size: 1024, include: () => true } as const;

describe('buildTileIndex', () => {
  const index = buildTileIndex(roads, bounds, detail);

  it('puts every road in exactly one tile', () => {
    const all = [...index.values()].flatMap((entry) => [...entry.roads]);
    expect(all.sort((a, b) => a - b)).toEqual([0, 1, 2, 3, 4]);
  });

  it('owns a road by its bbox centre and covers its extension', () => {
    const tile = index.get('detail:0:0');
    const owner = index.get('detail:1:0');
    expect(tile?.roads).toEqual(Uint32Array.from([0, 4]));
    expect(owner?.roads).toEqual(Uint32Array.from([1, 2]));
    expect(owner?.bounds).toEqual({ minX: 1000, minY: 50, maxX: 1200, maxY: 100 });
    expect(owner?.cx).toBe(1536);
    expect(owner?.cy).toBe(512);
  });

  it('floors negative coordinates', () => {
    expect(index.get('detail:-1:-1')?.roads).toEqual(Uint32Array.from([3]));
  });

  it('excludes residential roads from the city band', () => {
    const city = {
      name: 'city',
      size: 4096,
      include: (road: number) => (RANKS[roads.classCode[road] ?? 0] ?? 0) >= 10,
    } as const;
    const cityIndex = buildTileIndex(roads, bounds, city);
    const all = [...cityIndex.values()].flatMap((entry) => [...entry.roads]);
    expect(all).not.toContain(2);
    expect(all.sort((a, b) => a - b)).toEqual([0, 1, 3, 4]);
  });
});
