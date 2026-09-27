import type { MapMeta } from '../../generated/MapMeta';
import { screenToWorld, type Camera } from '../camera';
import type { Layers } from '../layers';
import type { Rect } from '../rect';
import type { RoadStore } from '../road-store';
import { cityStyle } from './road-style';
import { STREET_MIN_SCALE } from '../style';
import { buildMarkings } from './markings-builder';
import type { StreetData } from './street-data';
import { BuildingTiles } from './building-tiles';
import { buildTile } from './tile-builder';
import type { BandName, TileEntry } from './tile-index';
import {
  cityInclude,
  countShadowPieces,
  createBand,
  createDetailBand,
  markDirty,
  prioritized,
  type BandState,
} from './band-state';
import { moveRoads, moveStreet, truncateIndex, type NetworkChange } from './tile-invalidate';
import { visibleTiles } from './visible';

export { cityInclude } from './band-state';

export const CITY_MAX_SCALE = 0.35;

export interface TileSources extends StreetData {
  pickRoad: (x: number, y: number, tolerance: number) => number | undefined;
}

export interface ActiveTiles {
  band: BandName;
  keys: readonly string[];
  entry: (key: string) => TileEntry | undefined;
  width: (road: number) => number;
}
const CITY_TILE = 4096;
const VIEW_MARGIN = 256;
const BUILDS_PER_FRAME = 2;

function viewRect(camera: Camera, viewW: number, viewH: number): Rect {
  const [minX, minY] = screenToWorld(camera, 0, 0);
  const [maxX, maxY] = screenToWorld(camera, viewW, viewH);
  return { minX, minY, maxX, maxY };
}

export class TileManager {
  visibleCount = 0;
  builtCount = 0;
  markingsBuilt = 0;
  buildingsBuilt = 0;
  shadowPieces = 0;
  activeBand: BandName = 'city';
  private visibleKeys: readonly string[] = [];
  private readonly city: BandState;
  private readonly detail: BandState;
  private readonly buildings: BuildingTiles;

  constructor(
    private readonly roads: RoadStore,
    meta: MapMeta,
    private readonly layers: Layers,
    street: TileSources,
  ) {
    const city = { name: 'city', size: CITY_TILE, include: cityInclude(roads, meta) } as const;
    this.city = createBand(roads, city, cityStyle(meta.class_ranks));
    this.detail = createDetailBand(roads, street);
    const sources = { ...street, roads, classNames: meta.class_names };
    this.buildings = new BuildingTiles(sources, layers.buildings);
  }

  activeTiles(): ActiveTiles {
    const band = this.activeBand === 'city' ? this.city : this.detail;
    return {
      band: band.name,
      keys: this.visibleKeys,
      entry: (key) => band.index.get(key),
      width: (road) => band.style(this.roads, road, 'fill').width,
    };
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
    markDirty(this.detail, this.buildings.near(this.detail.index, change.roads));
  }

  update(camera: Camera, viewW: number, viewH: number): void {
    const cityActive = camera.scale < CITY_MAX_SCALE;
    const active = cityActive ? this.city : this.detail;
    const inactive = cityActive ? this.detail : this.city;
    this.activeBand = active.name;
    const keys = visibleTiles(active.index.entries, viewRect(camera, viewW, viewH), VIEW_MARGIN);
    const built = this.buildPending(active, keys);
    active.cache.show(keys);
    const remaining = active.cache.pending(keys).length;
    if (remaining === 0) {
      inactive.cache.show([]);
    }
    this.visibleKeys = keys;
    this.visibleCount = keys.length;
    this.builtCount = keys.length - remaining;
    const street = camera.scale >= STREET_MIN_SCALE;
    this.updateMarkings(active, keys, street);
    this.buildings.setVisible(street);
    if (street && !cityActive) {
      this.buildings.ensure(active, keys, BUILDS_PER_FRAME - built);
    }
    this.buildingsBuilt = keys.filter(
      (key) => active.cache.get(key)?.buildings !== undefined,
    ).length;
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

  private buildPending(band: BandState, keys: readonly string[]): number {
    const batch = prioritized(band, band.cache.pending(keys)).slice(0, BUILDS_PER_FRAME);
    for (const key of batch) {
      this.buildOne(band, key);
    }
    return batch.length;
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
