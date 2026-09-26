import { describe, expect, it } from 'vitest';
import type { Rect } from '../rect';
import type { TileEntry, TileIndex } from './tile-index';
import { visibleTiles } from './visible';

function entry(bounds: Rect, cx: number, cy: number): TileEntry {
  return { roads: new Uint32Array(), bounds, cx, cy };
}

const index: TileIndex = new Map([
  ['b', entry({ minX: 0, minY: 0, maxX: 100, maxY: 100 }, 50, 50)],
  ['a', entry({ minX: 100, minY: 0, maxX: 200, maxY: 100 }, 150, 50)],
  ['c', entry({ minX: 250, minY: 0, maxX: 300, maxY: 100 }, 275, 50)],
  ['d', entry({ minX: 400, minY: 0, maxX: 500, maxY: 100 }, 450, 50)],
]);

const view = { minX: 0, minY: 0, maxX: 200, maxY: 100 };

describe('visibleTiles', () => {
  it('keeps tiles within the margin', () => {
    expect(visibleTiles(index, view, 0)).toEqual(['a', 'b']);
    expect(visibleTiles(index, view, 60)).toEqual(['a', 'b', 'c']);
  });

  it('orders by distance from the view centre', () => {
    const shifted = { minX: 150, minY: 0, maxX: 350, maxY: 100 };
    expect(visibleTiles(index, shifted, 100)).toEqual(['c', 'a', 'b', 'd']);
  });
});
