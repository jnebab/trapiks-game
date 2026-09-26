import type { MarkerChange } from '../detail-store';
import type { RoadStore } from '../road-store';
import type { StreetData } from './street-data';
import { placeJunction, placeMarker, placeRoad, type TileIndex } from './tile-index';

export interface NetworkChange {
  roads: readonly number[];
  nodes: readonly number[];
  markers: MarkerChange;
  setbackRoads: readonly number[];
}

function addKey(dirty: Set<string>, key: string | undefined): void {
  if (key !== undefined) {
    dirty.add(key);
  }
}

export function moveRoads(index: TileIndex, roads: RoadStore, ids: readonly number[]): Set<string> {
  const dirty = new Set<string>();
  for (const road of ids) {
    addKey(dirty, index.ownerOf('roads', road));
    addKey(dirty, placeRoad(index, roads, road));
  }
  return dirty;
}

export function moveStreet(
  index: TileIndex,
  street: StreetData,
  change: NetworkChange,
): Set<string> {
  const dirty = new Set<string>();
  for (const node of change.nodes) {
    addKey(dirty, index.ownerOf('junctions', node));
    addKey(dirty, placeJunction(index, street, node));
  }
  for (const id of [...change.markers.removed, ...change.markers.added]) {
    addKey(dirty, index.ownerOf('markers', id));
    addKey(dirty, placeMarker(index, street, id));
  }
  for (const road of change.setbackRoads) {
    addKey(dirty, index.ownerOf('roads', road));
  }
  return dirty;
}

export function truncateIndex(
  index: TileIndex,
  counts: { roads: number; nodes: number },
  markers: readonly number[],
): Set<string> {
  const dirty = new Set<string>();
  for (const road of index.ids('roads').filter((id) => id >= counts.roads)) {
    addKey(dirty, index.remove('roads', road));
  }
  for (const node of index.ids('junctions').filter((id) => id >= counts.nodes)) {
    addKey(dirty, index.remove('junctions', node));
  }
  for (const id of markers) {
    addKey(dirty, index.remove('markers', id));
  }
  return dirty;
}
