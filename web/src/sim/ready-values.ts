import type { MapMeta } from '../generated/MapMeta';
import { isGameMode, isRegion } from './game-values';
import { areaShape, nodeShape, roadShape } from './map-arrays';
import { junctionShape, markerShape, signalPillShape } from './street-arrays';
import { hasArrays, isRecord } from './values';

export function isMapMeta(value: unknown): value is MapMeta {
  if (!isRecord(value)) {
    return false;
  }
  return (
    typeof value.map_hash === 'string' &&
    typeof value.road_count === 'number' &&
    typeof value.node_count === 'number' &&
    typeof value.area_ring_count === 'number' &&
    Array.isArray(value.class_ranks) &&
    Array.isArray(value.bounds)
  );
}

function hasMapArrays(value: Record<string, unknown>): boolean {
  return (
    hasArrays(value.roads, roadShape) &&
    hasArrays(value.nodes, nodeShape) &&
    hasArrays(value.areas, areaShape) &&
    value.roadSetbacks instanceof Float32Array
  );
}

function hasStreetArrays(value: Record<string, unknown>): boolean {
  return (
    hasArrays(value.junctions, junctionShape) &&
    hasArrays(value.markers, markerShape) &&
    hasArrays(value.signalPills, signalPillShape)
  );
}

export function isReadyMessage(value: Record<string, unknown>): boolean {
  const regionOk = value.region === null || isRegion(value.region);
  return (
    isGameMode(value.mode) &&
    regionOk &&
    isMapMeta(value.meta) &&
    hasMapArrays(value) &&
    hasStreetArrays(value)
  );
}
