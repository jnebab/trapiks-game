import { describe, expect, it } from 'vitest';
import type { RoadArrays } from '../../sim/protocol';
import { DetailStore } from '../detail-store';
import { NodeStore } from '../node-store';
import { RoadStore } from '../road-store';
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
    from: new Uint32Array(5),
    to: new Uint32Array(5),
  };
}

const roads = RoadStore.fromArrays(makeRoads());
const detailBand = { name: 'detail', size: 1024, include: () => true } as const;

describe('buildTileIndex', () => {
  const index = buildTileIndex(roads, detailBand);

  it('puts every road in exactly one tile', () => {
    const all = [...index.entries.values()].flatMap((entry) => [...entry.roads]);
    expect(all.sort((a, b) => a - b)).toEqual([0, 1, 2, 3, 4]);
  });

  it('owns a road by its bbox centre and covers its extension', () => {
    const tile = index.get('detail:0:0');
    const owner = index.get('detail:1:0');
    expect(tile?.roads).toEqual([0, 4]);
    expect(owner?.roads).toEqual([1, 2]);
    expect(owner?.bounds).toEqual({ minX: 1000, minY: 50, maxX: 1200, maxY: 100 });
    expect(owner?.cx).toBe(1536);
    expect(owner?.cy).toBe(512);
  });

  it('floors negative coordinates', () => {
    expect(index.get('detail:-1:-1')?.roads).toEqual([3]);
  });

  it('excludes residential roads from the city band', () => {
    const city = {
      name: 'city',
      size: 4096,
      include: (road: number) => (RANKS[roads.classCode(road)] ?? 0) >= 10,
    } as const;
    const cityIndex = buildTileIndex(roads, city);
    const all = [...cityIndex.entries.values()].flatMap((entry) => [...entry.roads]);
    expect(all).not.toContain(2);
    expect(all.sort((a, b) => a - b)).toEqual([0, 1, 3, 4]);
  });
});

describe('buildTileIndex street detail', () => {
  const nodes = NodeStore.fromArrays({
    x: Float32Array.from([1100, 10]),
    y: Float32Array.from([60, 10]),
    controlCode: new Uint8Array(2),
  });
  const detail = DetailStore.fromArrays(
    new Float32Array(10),
    {
      node: Uint32Array.from([0]),
      layer: new Int8Array(1),
      minLayer: new Int8Array(1),
      ringStart: Uint32Array.from([0, 2]),
      x: Float32Array.from([1090, 1110]),
      y: Float32Array.from([40, 70]),
    },
    {
      link: Uint32Array.from([0]),
      node: Uint32Array.from([1]),
      kind: Uint8Array.from([1]),
      x1: Float32Array.from([-20]),
      y1: Float32Array.from([5]),
      x2: Float32Array.from([-4]),
      y2: Float32Array.from([5]),
    },
  );
  const index = buildTileIndex(roads, detailBand, { detail, nodes });

  it('owns junctions by node and markers by midpoint', () => {
    expect(index.get('detail:1:0')?.junctions).toEqual([0]);
    expect(index.get('detail:-1:0')?.markers).toEqual([0]);
    expect(index.get('detail:-1:0')?.bounds.minX).toBe(-20);
  });
});
