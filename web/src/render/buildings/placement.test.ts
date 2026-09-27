import { describe, expect, it } from 'vitest';
import type { RoadClass } from '../../generated/RoadClass';
import { Picking } from '../../edit/picking';
import { DetailStore } from '../detail-store';
import { NodeStore } from '../node-store';
import { RoadStore } from '../road-store';
import { LotGrid } from './lot-grid';
import { corners, distanceTo, overlaps } from './obb';
import { placeLots, type Lot, type LotStore } from './placement';

const CLASSES: readonly RoadClass[] = ['Residential', 'Primary'];
const XS = [0, 300, 150, 150];
const YS = [0, 0, -200, 200];

function network(): { roads: RoadStore; nodes: NodeStore } {
  const roads = RoadStore.fromArrays({
    pointStart: Uint32Array.from([0, 2, 4]),
    x: Float32Array.from(XS),
    y: Float32Array.from(YS),
    classCode: Uint8Array.from([0, 1]),
    lanesForward: Uint8Array.from([1, 2]),
    lanesBackward: Uint8Array.from([1, 2]),
    layer: new Int8Array(2),
    name: new Uint32Array(2),
    roundabout: new Uint8Array(2),
    from: Uint32Array.from([0, 2]),
    to: Uint32Array.from([1, 3]),
  });
  const nodes = NodeStore.fromArrays({
    x: Float32Array.from(XS),
    y: Float32Array.from(YS),
    controlCode: new Uint8Array(4),
  });
  return { roads, nodes };
}

function store(junction?: [number, number]): LotStore {
  const { roads, nodes } = network();
  const picking = new Picking(roads, nodes, new DetailStore());
  return {
    roads,
    classNames: CLASSES,
    hitsRoad: (x, y) => picking.pickRoad(x, y, 2) !== undefined,
    nearJunction: (box) => junction !== undefined && distanceTo(box, ...junction) <= 20,
    grid: new LotGrid(),
  };
}

function footprints(lots: readonly Lot[]) {
  return lots.flatMap((lot) => (lot.parking === undefined ? [lot.body] : [lot.body, lot.parking]));
}

describe('placeLots', () => {
  it('is deterministic for a road side', () => {
    expect(placeLots(0, 0, store())).toEqual(placeLots(0, 0, store()));
    expect(placeLots(0, 0, store())).not.toEqual(placeLots(0, 1, store()));
  });

  it('keeps every lot off the roads', () => {
    const context = store();
    const lots = [0, 1].flatMap((road) => [0, 1].flatMap((side) => placeLots(road, side, context)));
    expect(lots.length).toBeGreaterThan(10);
    const probe = store();
    for (const box of footprints(lots)) {
      for (const [x, y] of [...corners(box), [box.cx, box.cy] as const]) {
        expect(probe.hitsRoad(x, y)).toBe(false);
      }
    }
  });

  it('never overlaps lots on the same side', () => {
    const boxes = footprints(placeLots(1, 0, store()));
    boxes.forEach((a, i) => {
      boxes.slice(i + 1).forEach((b) => {
        expect(overlaps(a, b)).toBe(false);
      });
    });
  });

  it('keeps clear of junctions', () => {
    const lots = placeLots(0, 0, store([150, 0]));
    expect(lots.length).toBeGreaterThan(0);
    for (const lot of lots) {
      expect(distanceTo(lot.body, 150, 0)).toBeGreaterThan(20);
    }
  });
});
