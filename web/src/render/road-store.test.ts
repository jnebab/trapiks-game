import { describe, expect, it } from 'vitest';
import { RoadStore, type RoadRows } from './road-store';

function initial(): RoadStore {
  return RoadStore.fromArrays({
    pointStart: Uint32Array.from([0, 2, 5]),
    x: Float32Array.from([0, 10, 20, 30, 40]),
    y: Float32Array.from([0, 0, 5, 5, 5]),
    classCode: Uint8Array.from([3, 4]),
    lanesForward: Uint8Array.from([1, 2]),
    lanesBackward: Uint8Array.from([1, 0]),
    layer: Int8Array.from([0, 1]),
    name: Uint32Array.from([7, 8]),
    from: Uint32Array.from([0, 2]),
    to: Uint32Array.from([1, 3]),
  });
}

function row(id: number, points: number[][], lanes: [number, number]): RoadRows {
  return {
    ids: Uint32Array.from([id]),
    classCode: Uint8Array.from([5]),
    lanesForward: Uint8Array.from([lanes[0]]),
    lanesBackward: Uint8Array.from([lanes[1]]),
    layer: Int8Array.from([2]),
    name: Uint32Array.from([9]),
    deleted: new Uint8Array(1),
    from: Uint32Array.from([4]),
    to: Uint32Array.from([5]),
    pointLen: Uint32Array.from([points.length]),
    x: Float32Array.from(points.map((p) => p[0] ?? 0)),
    y: Float32Array.from(points.map((p) => p[1] ?? 0)),
  };
}

describe('RoadStore', () => {
  it('loads columns and point ranges', () => {
    const store = initial();
    expect(store.count).toBe(2);
    expect(store.pointRange(1)).toEqual([2, 5]);
    expect(store.pointsOf(0)).toEqual([
      { x: 0, y: 0 },
      { x: 10, y: 0 },
    ]);
    expect([store.lanes(1), store.layer(1), store.name(0), store.from(1), store.to(1)]).toEqual([
      2, 1, 7, 2, 3,
    ]);
  });
});

describe('RoadStore updates', () => {
  it('updates an existing row with new geometry', () => {
    const store = initial();
    store.applyRoads(
      row(
        0,
        [
          [1, 1],
          [2, 2],
          [3, 3],
        ],
        [3, 3],
      ),
    );
    expect(store.count).toBe(2);
    expect(store.lanes(0)).toBe(6);
    expect(store.pointsOf(0)).toEqual([
      { x: 1, y: 1 },
      { x: 2, y: 2 },
      { x: 3, y: 3 },
    ]);
    expect(store.pointsOf(1).length).toBe(3);
  });
});

describe('RoadStore growth', () => {
  it('appends past capacity and truncates', () => {
    const store = initial();
    store.applyRoads(
      row(
        40,
        [
          [5, 5],
          [6, 6],
        ],
        [1, 0],
      ),
    );
    expect(store.count).toBe(41);
    expect(store.pointsOf(40)).toEqual([
      { x: 5, y: 5 },
      { x: 6, y: 6 },
    ]);
    store.truncate(2);
    expect(store.count).toBe(2);
    expect(store.isDeleted(40)).toBe(true);
  });

  it('flags deleted rows', () => {
    const store = initial();
    store.applyRoads({
      ...row(
        1,
        [
          [0, 0],
          [1, 0],
        ],
        [1, 1],
      ),
      deleted: Uint8Array.from([1]),
    });
    expect(store.isDeleted(1)).toBe(true);
    expect(store.isDeleted(0)).toBe(false);
  });
});
