import type { RoadArrays } from '../../sim/protocol';
import type { Rect } from '../rect';
import { boundsOf } from './road-bounds';
import { junctionOwner, markerOwner, type Owned, type StreetData } from './street-data';
import { tileKey, tileOf } from './tile-key';

export type BandName = 'city' | 'detail';

export interface Band {
  name: BandName;
  size: number;
  include: (road: number) => boolean;
}

export interface TileEntry {
  roads: Uint32Array;
  junctions: Uint32Array;
  markers: Uint32Array;
  bounds: Rect;
  cx: number;
  cy: number;
}

export type TileIndex = Map<string, TileEntry>;

type ItemKind = 'roads' | 'junctions' | 'markers';

type Draft = Record<ItemKind, number[]> & { bounds: Rect; cx: number; cy: number };

function union(a: Rect, b: Rect): Rect {
  return {
    minX: Math.min(a.minX, b.minX),
    minY: Math.min(a.minY, b.minY),
    maxX: Math.max(a.maxX, b.maxX),
    maxY: Math.max(a.maxY, b.maxY),
  };
}

function addItem(drafts: Map<string, Draft>, band: Band, owned: Owned, item: [ItemKind, number]) {
  const [tx, ty] = tileOf(band.size, owned.x, owned.y);
  const key = tileKey(band.name, tx, ty);
  const cx = (tx + 0.5) * band.size;
  const cy = (ty + 0.5) * band.size;
  const draft = drafts.get(key) ?? {
    roads: [],
    junctions: [],
    markers: [],
    bounds: owned.box,
    cx,
    cy,
  };
  drafts.set(key, draft);
  draft[item[0]].push(item[1]);
  draft.bounds = union(draft.bounds, owned.box);
}

function roadOwner(bounds: Float32Array, road: number): Owned {
  const box = boundsOf(bounds, road);
  return { x: (box.minX + box.maxX) / 2, y: (box.minY + box.maxY) / 2, box };
}

function addStreet(drafts: Map<string, Draft>, band: Band, street: StreetData): void {
  for (let shape = 0; shape < street.junctions.node.length; shape += 1) {
    addItem(drafts, band, junctionOwner(street, shape), ['junctions', shape]);
  }
  for (let marker = 0; marker < street.markers.link.length; marker += 1) {
    addItem(drafts, band, markerOwner(street, marker), ['markers', marker]);
  }
}

export function buildTileIndex(
  roads: RoadArrays,
  bounds: Float32Array,
  band: Band,
  street?: StreetData,
): TileIndex {
  const drafts = new Map<string, Draft>();
  for (let road = 0; road < roads.layer.length; road += 1) {
    if (band.include(road)) {
      addItem(drafts, band, roadOwner(bounds, road), ['roads', road]);
    }
  }
  if (street !== undefined) {
    addStreet(drafts, band, street);
  }
  const index: TileIndex = new Map();
  for (const [key, draft] of drafts) {
    index.set(key, {
      ...draft,
      roads: Uint32Array.from(draft.roads),
      junctions: Uint32Array.from(draft.junctions),
      markers: Uint32Array.from(draft.markers),
    });
  }
  return index;
}
