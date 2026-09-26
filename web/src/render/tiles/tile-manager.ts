import type { MapMeta } from '../../generated/MapMeta';
import { screenToWorld, type Camera } from '../camera';
import type { Layers } from '../layers';
import type { Rect } from '../rect';
import type { RoadStore } from '../road-store';
import { cityStyle, detailStyle, type RoadStyle } from './road-style';
import { STREET_MIN_SCALE } from '../style';
import { buildMarkings } from './markings-builder';
import type { StreetData } from './street-data';
import { buildTile } from './tile-builder';
import { TileCache } from './tile-cache';
import { buildTileIndex, type Band, type BandName, type TileIndex } from './tile-index';
import { moveRoads, moveStreet, truncateIndex, type NetworkChange } from './tile-invalidate';
import { visibleTiles } from './visible';

export const CITY_MAX_SCALE = 0.35;
const CITY_TILE = 4096;
const DETAIL_TILE = 512;
const VIEW_MARGIN = 256;
const BUILDS_PER_FRAME = 2;

interface BandState {
  name: BandName;
  index: TileIndex;
  cache: TileCache;
  style: RoadStyle;
  street?: StreetData;
  dirty: Set<string>;
}

function viewRect(camera: Camera, viewW: number, viewH: number): Rect {
  const [minX, minY] = screenToWorld(camera, 0, 0);
  const [maxX, maxY] = screenToWorld(camera, viewW, viewH);
  return { minX, minY, maxX, maxY };
}

function cityInclude(roads: RoadStore, meta: MapMeta): (road: number) => boolean {
  const primaryRank = meta.class_ranks[meta.class_names.indexOf('Primary')] ?? 0;
  return (road) => (meta.class_ranks[roads.classCode(road)] ?? 0) >= primaryRank;
}

function createBand(roads: RoadStore, band: Band, style: RoadStyle): BandState {
  return {
    name: band.name,
    index: buildTileIndex(roads, band),
    cache: new TileCache(),
    style,
    dirty: new Set(),
  };
}

function createDetailBand(roads: RoadStore, street: StreetData): BandState {
  const band = { name: 'detail', size: DETAIL_TILE, include: () => true } as const;
  return {
    name: band.name,
    index: buildTileIndex(roads, band, street),
    cache: new TileCache(),
    style: detailStyle,
    street,
    dirty: new Set(),
  };
}

function markDirty(band: BandState, keys: Set<string>): void {
  for (const key of keys) {
    band.cache.drop(key);
    band.dirty.add(key);
  }
}

function prioritized(band: BandState, pending: readonly string[]): string[] {
  const dirty = pending.filter((key) => band.dirty.has(key));
  const fresh = pending.filter((key) => !band.dirty.has(key));
  return [...dirty, ...fresh];
}

function countShadowPieces(band: BandState, keys: readonly string[]): number {
  let count = 0;
  for (const key of keys) {
    const pieces = band.cache.get(key)?.pieces ?? [];
    count += pieces.filter((piece) => piece.pass === 'shadow').length;
  }
  return count;
}

export class TileManager {
  visibleCount = 0;
  builtCount = 0;
  markingsBuilt = 0;
  shadowPieces = 0;
  activeBand: BandName = 'city';
  private readonly city: BandState;
  private readonly detail: BandState;

  constructor(
    private readonly roads: RoadStore,
    meta: MapMeta,
    private readonly layers: Layers,
    street: StreetData,
  ) {
    const city = { name: 'city', size: CITY_TILE, include: cityInclude(roads, meta) } as const;
    this.city = createBand(roads, city, cityStyle(meta.class_ranks));
    this.detail = createDetailBand(roads, street);
  }

  truncate(counts: { roads: number; nodes: number }, markers: readonly number[]): void {
    for (const band of [this.city, this.detail]) {
      markDirty(band, truncateIndex(band.index, counts, markers));
    }
  }

  invalidate(change: NetworkChange): void {
    for (const band of [this.city, this.detail]) {
      markDirty(band, moveRoads(band.index, this.roads, change.roads));
      if (band.street !== undefined) {
        markDirty(band, moveStreet(band.index, band.street, change));
      }
    }
  }

  update(camera: Camera, viewW: number, viewH: number): void {
    const cityActive = camera.scale < CITY_MAX_SCALE;
    const active = cityActive ? this.city : this.detail;
    const inactive = cityActive ? this.detail : this.city;
    this.activeBand = active.name;
    const keys = visibleTiles(active.index.entries, viewRect(camera, viewW, viewH), VIEW_MARGIN);
    this.buildPending(active, keys);
    active.cache.show(keys);
    const remaining = active.cache.pending(keys).length;
    if (remaining === 0) {
      inactive.cache.show([]);
    }
    this.visibleCount = keys.length;
    this.builtCount = keys.length - remaining;
    this.updateMarkings(active, keys, camera.scale >= STREET_MIN_SCALE);
    this.layers.updateShadows(!cityActive, camera.scale);
    this.shadowPieces = countShadowPieces(active, keys);
  }

  private updateMarkings(band: BandState, keys: readonly string[], street: boolean): void {
    this.layers.showMarkings(street);
    if (street) {
      keys.forEach((key) => {
        this.ensureMarkings(band, key);
      });
    }
    this.markingsBuilt = keys.filter((key) => band.cache.get(key)?.markings === true).length;
  }

  private ensureMarkings(band: BandState, key: string): void {
    const tile = band.cache.get(key);
    const entry = band.index.get(key);
    if (tile === undefined || tile.markings || entry === undefined || band.street === undefined) {
      return;
    }
    const pieces = buildMarkings(entry, this.roads, band.street);
    for (const piece of pieces) {
      this.layers.road(piece.layer, piece.pass).addChild(piece.graphics);
    }
    band.cache.addMarkings(key, pieces);
  }

  private buildPending(band: BandState, keys: readonly string[]): void {
    for (const key of prioritized(band, band.cache.pending(keys)).slice(0, BUILDS_PER_FRAME)) {
      this.buildOne(band, key);
    }
  }

  private buildOne(band: BandState, key: string): void {
    band.dirty.delete(key);
    const entry = band.index.get(key);
    if (entry === undefined) {
      return;
    }
    const tile = buildTile(entry, this.roads, band.style, band.street);
    for (const piece of tile.pieces) {
      this.layers.road(piece.layer, piece.pass).addChild(piece.graphics);
    }
    band.cache.insert(key, tile);
  }
}
