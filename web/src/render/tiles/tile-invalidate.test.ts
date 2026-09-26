import { describe, expect, it } from 'vitest';
import { RoadStore } from '../road-store';
import { buildTileIndex, type Band } from './tile-index';
import { moveRoads } from './tile-invalidate';

function store(): RoadStore {
  return RoadStore.fromArrays({
    pointStart: Uint32Array.from([0, 2, 4]),
    x: Float32Array.from([10, 100, 5000, 5100]),
    y: Float32Array.from([10, 10, 50, 50]),
    classCode: new Uint8Array(2),
    lanesForward: Uint8Array.from([1, 1]),
    lanesBackward: Uint8Array.from([1, 1]),
    layer: new Int8Array(2),
    name: new Uint32Array(2),
    roundabout: new Uint8Array(2),
    from: Uint32Array.from([0, 2]),
    to: Uint32Array.from([1, 3]),
  });
}

function moveFirstRoad(roads: RoadStore, deleted = 0): void {
  roads.applyRoads({
    ids: Uint32Array.from([0]),
    classCode: new Uint8Array(1),
    lanesForward: Uint8Array.from([1]),
    lanesBackward: Uint8Array.from([1]),
    layer: new Int8Array(1),
    name: new Uint32Array(1),
    roundabout: new Uint8Array(1),
    deleted: Uint8Array.from([deleted]),
    from: Uint32Array.from([0]),
    to: Uint32Array.from([1]),
    pointLen: Uint32Array.from([2]),
    x: Float32Array.from([9000, 9100]),
    y: Float32Array.from([9000, 9000]),
  });
}

const bands: Band[] = [
  { name: 'city', size: 4096, include: () => true },
  { name: 'detail', size: 512, include: () => true },
];

describe('moveRoads', () => {
  it.each(bands)('moves a road between tiles in the $name band', (band) => {
    const roads = store();
    const index = buildTileIndex(roads, band);
    const before = index.ownerOf('roads', 0);
    moveFirstRoad(roads);
    const dirty = moveRoads(index, roads, [0]);
    const after = index.ownerOf('roads', 0);
    expect(after).not.toBe(before);
    expect([...dirty].sort()).toEqual([before, after].sort());
    expect(index.get(after ?? '')?.roads).toEqual([0]);
    expect(index.get(before ?? '')?.roads ?? []).not.toContain(0);
    expect(index.get(after ?? '')?.bounds).toEqual({
      minX: 9000,
      minY: 9000,
      maxX: 9100,
      maxY: 9000,
    });
  });

  it.each(bands)('drops a deleted road from the $name band', (band) => {
    const roads = store();
    const index = buildTileIndex(roads, band);
    const before = index.ownerOf('roads', 0);
    moveFirstRoad(roads, 1);
    expect([...moveRoads(index, roads, [0])]).toEqual([before]);
    expect(index.ownerOf('roads', 0)).toBeUndefined();
    expect(index.get(before ?? '')).toBeUndefined();
  });
});
