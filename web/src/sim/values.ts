import type { SimConfig } from '../generated/SimConfig';
import type { StatsSnapshot } from '../generated/StatsSnapshot';

export type Speed = 0 | 1 | 2 | 4 | 8 | 'max';

export const SPEEDS: readonly Speed[] = [0, 1, 2, 4, 8, 'max'];

export interface SnapshotBuffers {
  ids: Uint32Array<ArrayBuffer>;
  x: Float32Array<ArrayBuffer>;
  y: Float32Array<ArrayBuffer>;
  heading: Float32Array<ArrayBuffer>;
  style: Uint8Array<ArrayBuffer>;
  layer: Int8Array<ArrayBuffer>;
}

export type ArrayCtor =
  Uint8ArrayConstructor | Int8ArrayConstructor | Uint32ArrayConstructor | Float32ArrayConstructor;

const bufferShape: Record<keyof SnapshotBuffers, ArrayCtor> = {
  ids: Uint32Array,
  x: Float32Array,
  y: Float32Array,
  heading: Float32Array,
  style: Uint8Array,
  layer: Int8Array,
};

const statsNumbers: readonly (keyof StatsSnapshot)[] = [
  'tick',
  'sim_time_s',
  'active',
  'spawned',
  'arrivals',
  'mean_speed',
];

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

export function hasArrays(value: unknown, shape: Record<string, ArrayCtor>): boolean {
  if (!isRecord(value)) {
    return false;
  }
  return Object.entries(shape).every(([key, ctor]) => value[key] instanceof ctor);
}

export function isSpeed(value: unknown): value is Speed {
  return SPEEDS.some((speed) => speed === value);
}

export function isSnapshotBuffers(value: unknown): value is SnapshotBuffers {
  return hasArrays(value, bufferShape);
}

export function snapshotTransfer(buffers: SnapshotBuffers): ArrayBuffer[] {
  return [buffers.ids, buffers.x, buffers.y, buffers.heading, buffers.style, buffers.layer].map(
    (a) => a.buffer,
  );
}

export function isStatsSnapshot(value: unknown): value is StatsSnapshot {
  if (!isRecord(value)) {
    return false;
  }
  return statsNumbers.every((key) => typeof value[key] === 'number');
}

function isRegion(value: unknown): boolean {
  if (!isRecord(value) || !isRecord(value.Region)) {
    return false;
  }
  const region = value.Region;
  return ['center_x', 'center_y', 'radius'].every((key) => typeof region[key] === 'number');
}

export function isSimConfig(value: unknown): value is SimConfig {
  if (!isRecord(value)) {
    return false;
  }
  const modeOk = value.mode === 'City' || isRegion(value.mode);
  return modeOk && typeof value.seed === 'number' && typeof value.vehicles_per_hour === 'number';
}
