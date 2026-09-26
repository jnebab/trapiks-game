import type { MapMeta } from '../../generated/MapMeta';
import type { RoadArrays } from '../../sim/protocol';
import { screenToWorld, type Camera } from '../camera';
import type { Layers } from '../layers';
import type { Rect } from '../rect';
import { roadBounds } from './road-bounds';
import { cityStyle, detailStyle, type RoadStyle } from './road-style';
import { STREET_MIN_SCALE } from '../style';
import { buildMarkings } from './markings-builder';
import type { StreetData } from './street-data';
import { buildTile } from './tile-builder';
import { TileCache } from './tile-cache';
import { buildTileIndex, type Band, type BandName, type TileIndex } from './tile-index';
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
}

function viewRect(camera: Camera, viewW: number, viewH: number): Rect {
  const [minX, minY] = screenToWorld(camera, 0, 0);
  const [maxX, maxY] = screenToWorld(camera, viewW, viewH);
  return { minX, minY, maxX, maxY };
}

function cityInclude(roads: RoadArrays, meta: MapMeta): (road: number) => boolean {
  const primaryRank = meta.class_ranks[meta.class_names.indexOf('Primary')] ?? 0;
  return (road) => (meta.class_ranks[roads.classCode[road] ?? 0] ?? 0) >= primaryRank;
}

function createBand(
  roads: RoadArrays,
  bounds: Float32Array,
  band: Band,
  style: RoadStyle,
): BandState {
  return {
    name: band.name,
    index: buildTileIndex(roads, bounds, band),
    cache: new TileCache(),
    style,
  };
}

function createDetailBand(roads: RoadArrays, bounds: Float32Array, street: StreetData): BandState {
  const band = { name: 'detail', size: DETAIL_TILE, include: () => true } as const;
  return {
    name: band.name,
    index: buildTileIndex(roads, bounds, band, street),
    cache: new TileCache(),
    style: detailStyle,
    street,
  };
}

export class TileManager {
  visibleCount = 0;
  builtCount = 0;
  markingsBuilt = 0;
  activeBand: BandName = 'city';
  private readonly city: BandState;
  private readonly detail: BandState;

  constructor(
    private readonly roads: RoadArrays,
    meta: MapMeta,
    private readonly layers: Layers,
    street: StreetData,
  ) {
    const bounds = roadBounds(roads);
    const city = { name: 'city', size: CITY_TILE, include: cityInclude(roads, meta) } as const;
    this.city = createBand(roads, bounds, city, cityStyle(meta.class_ranks));
    this.detail = createDetailBand(roads, bounds, street);
  }

  update(camera: Camera, viewW: number, viewH: number): void {
    const cityActive = camera.scale < CITY_MAX_SCALE;
    const active = cityActive ? this.city : this.detail;
    const inactive = cityActive ? this.detail : this.city;
    this.activeBand = active.name;
    const keys = visibleTiles(active.index, viewRect(camera, viewW, viewH), VIEW_MARGIN);
    this.buildPending(active, keys);
    active.cache.show(keys);
    const remaining = active.cache.pending(keys).length;
    if (remaining === 0) {
      inactive.cache.show([]);
    }
    this.visibleCount = keys.length;
    this.builtCount = keys.length - remaining;
    this.updateMarkings(active, keys, camera.scale >= STREET_MIN_SCALE);
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
    for (const key of band.cache.pending(keys).slice(0, BUILDS_PER_FRAME)) {
      this.buildOne(band, key);
    }
  }

  private buildOne(band: BandState, key: string): void {
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
