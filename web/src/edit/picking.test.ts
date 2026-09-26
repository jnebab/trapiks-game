import { describe, expect, it } from 'vitest';
import { DetailStore } from '../render/detail-store';
import { NodeStore } from '../render/node-store';
import { RoadStore, type RoadRows } from '../render/road-store';
import { Picking } from './picking';

function roads(): RoadStore {
  return RoadStore.fromArrays({
    pointStart: Uint32Array.from([0, 2, 4, 6]),
    x: Float32Array.from([0, 100, 50, 50, 0, 100]),
    y: Float32Array.from([0, 0, -50, 50, 20, 20]),
    classCode: new Uint8Array(3),
    lanesForward: Uint8Array.from([1, 1, 2]),
    lanesBackward: Uint8Array.from([1, 1, 2]),
    layer: Int8Array.from([0, 1, 0]),
    name: new Uint32Array(3),
    from: Uint32Array.from([0, 2, 4]),
    to: Uint32Array.from([1, 3, 5]),
  });
}

function setup(): { store: RoadStore; picking: Picking } {
  const store = roads();
  const nodes = NodeStore.fromArrays({
    x: Float32Array.from([0, 100, 50, 50, 0, 100]),
    y: Float32Array.from([0, 0, -50, 50, 20, 20]),
    controlCode: new Uint8Array(6),
  });
  const detail = DetailStore.fromArrays(
    new Float32Array(6),
    {
      node: Uint32Array.from([1, 0]),
      layer: new Int8Array(2),
      minLayer: new Int8Array(2),
      ringStart: Uint32Array.from([0, 1, 2]),
      x: Float32Array.from([100, 0]),
      y: Float32Array.from([0, 0]),
    },
    {
      link: new Uint32Array(0),
      node: new Uint32Array(0),
      kind: new Uint8Array(0),
      x1: new Float32Array(0),
      y1: new Float32Array(0),
      x2: new Float32Array(0),
      y2: new Float32Array(0),
    },
  );
  return { store, picking: new Picking(store, nodes, detail) };
}

function moved(id: number, y: number, deleted = 0): RoadRows {
  return {
    ids: Uint32Array.from([id]),
    classCode: new Uint8Array(1),
    lanesForward: Uint8Array.from([2]),
    lanesBackward: Uint8Array.from([2]),
    layer: new Int8Array(1),
    name: new Uint32Array(1),
    deleted: Uint8Array.from([deleted]),
    from: Uint32Array.from([4]),
    to: Uint32Array.from([5]),
    pointLen: Uint32Array.from([2]),
    x: Float32Array.from([0, 100]),
    y: Float32Array.from([y, y]),
  };
}

describe('Picking', () => {
  it('picks the nearest road by edge distance', () => {
    const { picking } = setup();
    expect(picking.pickRoad(20, 3, 2)).toBe(0);
    expect(picking.pickRoad(20, 14, 2)).toBe(2);
    expect(picking.pickRoad(20, 11, 1)).toBeUndefined();
  });

  it('prefers the higher layer where roads cross', () => {
    const { picking } = setup();
    expect(picking.pickRoad(50, 0, 1)).toBe(1);
  });

  it('follows geometry updates and skips deleted roads', () => {
    const { store, picking } = setup();
    store.applyRoads(moved(2, 300));
    picking.update([2]);
    expect(picking.pickRoad(20, 20, 1)).toBeUndefined();
    expect(picking.pickRoad(20, 301, 1)).toBe(2);
    store.applyRoads(moved(2, 300, 1));
    picking.update([2]);
    expect(picking.pickRoad(20, 301, 1)).toBeUndefined();
  });

  it('picks the nearest junction node', () => {
    const { picking } = setup();
    expect(picking.pickNode(90, 5, 20)).toBe(1);
    expect(picking.pickNode(50, 0, 50)).toBe(0);
    expect(picking.pickNode(50, 40, 10)).toBeUndefined();
  });
});
