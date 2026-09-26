import type { MapMeta } from '../generated/MapMeta';
import type { SimConfig } from '../generated/SimConfig';
import type { StatsSnapshot } from '../generated/StatsSnapshot';
import {
  hasArrays,
  isRecord,
  isSimConfig,
  isSnapshotBuffers,
  isSpeed,
  isStatsSnapshot,
  type ArrayCtor,
  type SnapshotBuffers,
  type Speed,
} from './values';

export type { SnapshotBuffers, Speed } from './values';

export interface LoadMessage {
  type: 'load';
  url: string;
  config: SimConfig;
}

export interface SpeedMessage {
  type: 'speed';
  speed: Speed;
}

export interface BuffersMessage {
  type: 'buffers';
  buffers: SnapshotBuffers;
}

export type MainMessage = LoadMessage | SpeedMessage | BuffersMessage;

export interface RoadArrays {
  pointStart: Uint32Array;
  x: Float32Array;
  y: Float32Array;
  classCode: Uint8Array;
  lanesForward: Uint8Array;
  lanesBackward: Uint8Array;
  layer: Int8Array;
  name: Uint32Array;
}

export interface NodeArrays {
  x: Float32Array;
  y: Float32Array;
  controlCode: Uint8Array;
}

export interface AreaArrays {
  kindCode: Uint8Array;
  ringStart: Uint32Array;
  x: Float32Array;
  y: Float32Array;
}

export interface ReadyMessage {
  type: 'ready';
  meta: MapMeta;
  roads: RoadArrays;
  nodes: NodeArrays;
  areas: AreaArrays;
}

export interface ErrorMessage {
  type: 'error';
  message: string;
}

export interface SnapshotMessage {
  type: 'snapshot';
  tick: number;
  simTime: number;
  count: number;
  buffers: SnapshotBuffers;
}

export interface StatsMessage {
  type: 'stats';
  stats: StatsSnapshot;
  roadSpeedRatio: Float32Array<ArrayBuffer>;
}

export type WorkerMessage = ReadyMessage | ErrorMessage | SnapshotMessage | StatsMessage;

const roadShape: Record<keyof RoadArrays, ArrayCtor> = {
  pointStart: Uint32Array,
  x: Float32Array,
  y: Float32Array,
  classCode: Uint8Array,
  lanesForward: Uint8Array,
  lanesBackward: Uint8Array,
  layer: Int8Array,
  name: Uint32Array,
};

const nodeShape: Record<keyof NodeArrays, ArrayCtor> = {
  x: Float32Array,
  y: Float32Array,
  controlCode: Uint8Array,
};

const areaShape: Record<keyof AreaArrays, ArrayCtor> = {
  kindCode: Uint8Array,
  ringStart: Uint32Array,
  x: Float32Array,
  y: Float32Array,
};

export function isMapMeta(value: unknown): value is MapMeta {
  if (!isRecord(value)) {
    return false;
  }
  return (
    typeof value.map_hash === 'string' &&
    typeof value.road_count === 'number' &&
    typeof value.node_count === 'number' &&
    typeof value.area_ring_count === 'number' &&
    Array.isArray(value.bounds)
  );
}

function isReadyMessage(value: Record<string, unknown>): boolean {
  return (
    isMapMeta(value.meta) &&
    hasArrays(value.roads, roadShape) &&
    hasArrays(value.nodes, nodeShape) &&
    hasArrays(value.areas, areaShape)
  );
}

function isSnapshotMessage(value: Record<string, unknown>): boolean {
  return (
    typeof value.tick === 'number' &&
    typeof value.simTime === 'number' &&
    typeof value.count === 'number' &&
    isSnapshotBuffers(value.buffers)
  );
}

function isStatsMessage(value: Record<string, unknown>): boolean {
  return isStatsSnapshot(value.stats) && value.roadSpeedRatio instanceof Float32Array;
}

const workerGuards: Record<WorkerMessage['type'], (value: Record<string, unknown>) => boolean> = {
  ready: isReadyMessage,
  error: (value) => typeof value.message === 'string',
  snapshot: isSnapshotMessage,
  stats: isStatsMessage,
};

const mainGuards: Record<MainMessage['type'], (value: Record<string, unknown>) => boolean> = {
  load: (value) => typeof value.url === 'string' && isSimConfig(value.config),
  speed: (value) => isSpeed(value.speed),
  buffers: (value) => isSnapshotBuffers(value.buffers),
};

function matches(
  value: unknown,
  guards: Record<string, (v: Record<string, unknown>) => boolean>,
): boolean {
  if (!isRecord(value) || typeof value.type !== 'string') {
    return false;
  }
  const guard = guards[value.type];
  return guard !== undefined && Object.hasOwn(guards, value.type) && guard(value);
}

export function isWorkerMessage(value: unknown): value is WorkerMessage {
  return matches(value, workerGuards);
}

export function isMainMessage(value: unknown): value is MainMessage {
  return matches(value, mainGuards);
}
