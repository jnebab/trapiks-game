import type { RoadArrays } from '../../sim/protocol';
import type { Rect } from '../rect';
import { boundsOf } from './road-bounds';
import { tileKey, tileOf } from './tile-key';

export type BandName = 'city' | 'detail';

export interface Band {
  name: BandName;
  size: number;
  include: (road: number) => boolean;
}

export interface TileEntry {
  roads: Uint32Array;
  bounds: Rect;
  cx: number;
  cy: number;
}

export type TileIndex = Map<string, TileEntry>;

interface Draft {
  roads: number[];
  bounds: Rect;
  cx: number;
  cy: number;
}

function union(a: Rect, b: Rect): Rect {
  return {
    minX: Math.min(a.minX, b.minX),
    minY: Math.min(a.minY, b.minY),
    maxX: Math.max(a.maxX, b.maxX),
    maxY: Math.max(a.maxY, b.maxY),
  };
}

function addRoad(drafts: Map<string, Draft>, band: Band, road: number, box: Rect): void {
  const [tx, ty] = tileOf(band.size, (box.minX + box.maxX) / 2, (box.minY + box.maxY) / 2);
  const key = tileKey(band.name, tx, ty);
  const draft = drafts.get(key);
  if (draft !== undefined) {
    draft.roads.push(road);
    draft.bounds = union(draft.bounds, box);
    return;
  }
  const cx = (tx + 0.5) * band.size;
  const cy = (ty + 0.5) * band.size;
  drafts.set(key, { roads: [road], bounds: box, cx, cy });
}

export function buildTileIndex(roads: RoadArrays, bounds: Float32Array, band: Band): TileIndex {
  const drafts = new Map<string, Draft>();
  for (let road = 0; road < roads.layer.length; road += 1) {
    if (band.include(road)) {
      addRoad(drafts, band, road, boundsOf(bounds, road));
    }
  }
  const index: TileIndex = new Map();
  for (const [key, draft] of drafts) {
    index.set(key, { ...draft, roads: Uint32Array.from(draft.roads) });
  }
  return index;
}
