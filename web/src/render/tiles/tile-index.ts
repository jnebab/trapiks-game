import type { Rect } from '../rect';
import type { RoadStore } from '../road-store';
import { junctionOwner, markerOwner, roadOwner, type Owned, type StreetData } from './street-data';
import { tileKey, tileOf } from './tile-key';

export type BandName = 'city' | 'detail';

export interface Band {
  name: BandName;
  size: number;
  include: (road: number) => boolean;
}

export type ItemKind = 'roads' | 'junctions' | 'markers';

export interface TileEntry {
  roads: number[];
  junctions: number[];
  markers: number[];
  bounds: Rect;
  cx: number;
  cy: number;
}

export type TileEntries = ReadonlyMap<string, TileEntry>;

type ItemMaps<T> = Record<ItemKind, Map<number, T>>;

function itemMaps<T>(): ItemMaps<T> {
  return { roads: new Map(), junctions: new Map(), markers: new Map() };
}

function union(a: Rect, b: Rect): Rect {
  return {
    minX: Math.min(a.minX, b.minX),
    minY: Math.min(a.minY, b.minY),
    maxX: Math.max(a.maxX, b.maxX),
    maxY: Math.max(a.maxY, b.maxY),
  };
}

const KINDS: readonly ItemKind[] = ['roads', 'junctions', 'markers'];

export class TileIndex {
  readonly entries = new Map<string, TileEntry>();
  private readonly owners = itemMaps<string>();
  private readonly boxes = itemMaps<Rect>();

  constructor(readonly band: Band) {}

  get(key: string): TileEntry | undefined {
    return this.entries.get(key);
  }

  ownerOf(kind: ItemKind, id: number): string | undefined {
    return this.owners[kind].get(id);
  }

  ids(kind: ItemKind): number[] {
    return [...this.owners[kind].keys()];
  }

  place(kind: ItemKind, id: number, owned: Owned): string {
    this.remove(kind, id);
    const [tx, ty] = tileOf(this.band.size, owned.x, owned.y);
    const key = tileKey(this.band.name, tx, ty);
    const size = this.band.size;
    const entry = this.entries.get(key) ?? {
      roads: [],
      junctions: [],
      markers: [],
      bounds: owned.box,
      cx: (tx + 0.5) * size,
      cy: (ty + 0.5) * size,
    };
    this.entries.set(key, entry);
    entry[kind].push(id);
    entry.bounds = union(entry.bounds, owned.box);
    this.owners[kind].set(id, key);
    this.boxes[kind].set(id, owned.box);
    return key;
  }

  remove(kind: ItemKind, id: number): string | undefined {
    const key = this.owners[kind].get(id);
    const entry = key === undefined ? undefined : this.entries.get(key);
    this.owners[kind].delete(id);
    this.boxes[kind].delete(id);
    if (key === undefined || entry === undefined) {
      return key;
    }
    entry[kind] = entry[kind].filter((item) => item !== id);
    this.reunion(key, entry);
    return key;
  }

  private reunion(key: string, entry: TileEntry): void {
    const boxes = KINDS.flatMap((kind) =>
      entry[kind].map((id) => this.boxes[kind].get(id)).filter((box) => box !== undefined),
    );
    const [first, ...rest] = boxes;
    if (first === undefined) {
      this.entries.delete(key);
      return;
    }
    entry.bounds = rest.reduce(union, first);
  }
}

export function placeRoad(index: TileIndex, roads: RoadStore, road: number): string | undefined {
  if (roads.isDeleted(road) || !index.band.include(road)) {
    return index.remove('roads', road);
  }
  return index.place('roads', road, roadOwner(roads, road));
}

export function placeJunction(
  index: TileIndex,
  street: StreetData,
  node: number,
): string | undefined {
  const ring = street.detail.junctions.get(node);
  if (ring === undefined) {
    return index.remove('junctions', node);
  }
  return index.place('junctions', node, junctionOwner(street.nodes, ring));
}

export function placeMarker(index: TileIndex, street: StreetData, id: number): string | undefined {
  const marker = street.detail.markers.get(id);
  if (marker === undefined) {
    return index.remove('markers', id);
  }
  return index.place('markers', id, markerOwner(marker));
}

export function buildTileIndex(roads: RoadStore, band: Band, street?: StreetData): TileIndex {
  const index = new TileIndex(band);
  for (let road = 0; road < roads.count; road += 1) {
    placeRoad(index, roads, road);
  }
  if (street === undefined) {
    return index;
  }
  for (const node of street.detail.junctions.keys()) {
    placeJunction(index, street, node);
  }
  for (const id of street.detail.markers.keys()) {
    placeMarker(index, street, id);
  }
  return index;
}
