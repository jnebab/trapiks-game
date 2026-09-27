import type { MapMeta } from '../../generated/MapMeta';
import type { RoadStore } from '../road-store';
import type { RoadStyle } from './road-style';
import type { StreetData } from './street-data';
import { TileCache } from './tile-cache';
import { buildTileIndex, type Band, type BandName, type TileIndex } from './tile-index';
import { detailStyle } from './road-style';

const DETAIL_TILE = 512;

export interface BandState {
  name: BandName;
  index: TileIndex;
  cache: TileCache;
  style: RoadStyle;
  street?: StreetData;
  dirty: Set<string>;
}

export function cityInclude(roads: RoadStore, meta: MapMeta): (road: number) => boolean {
  const primaryRank = meta.class_ranks[meta.class_names.indexOf('Primary')] ?? 0;
  return (road) =>
    !roads.isRoundabout(road) && (meta.class_ranks[roads.classCode(road)] ?? 0) >= primaryRank;
}

export function createBand(roads: RoadStore, band: Band, style: RoadStyle): BandState {
  return {
    name: band.name,
    index: buildTileIndex(roads, band),
    cache: new TileCache(),
    style,
    dirty: new Set(),
  };
}

export function createDetailBand(roads: RoadStore, street: StreetData): BandState {
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

export function markDirty(band: BandState, keys: Set<string>): void {
  for (const key of keys) {
    band.cache.drop(key);
    band.dirty.add(key);
  }
}

export function prioritized(band: BandState, pending: readonly string[]): string[] {
  const dirty = pending.filter((key) => band.dirty.has(key));
  const fresh = pending.filter((key) => !band.dirty.has(key));
  return [...dirty, ...fresh];
}

export function countShadowPieces(band: BandState, keys: readonly string[]): number {
  let count = 0;
  for (const key of keys) {
    const pieces = band.cache.get(key)?.pieces ?? [];
    count += pieces.filter((piece) => piece.pass === 'shadow').length;
  }
  return count;
}
