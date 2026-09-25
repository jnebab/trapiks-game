import type { MapMeta } from '../generated/MapMeta';

export interface LoadMessage {
  type: 'load';
  url: string;
}

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

export type WorkerMessage = ReadyMessage | ErrorMessage;

type ArrayCtor =
  Uint8ArrayConstructor | Int8ArrayConstructor | Uint32ArrayConstructor | Float32ArrayConstructor;

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

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function hasArrays(value: unknown, shape: Record<string, ArrayCtor>): boolean {
  if (!isRecord(value)) {
    return false;
  }
  return Object.entries(shape).every(([key, ctor]) => value[key] instanceof ctor);
}

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

export function isWorkerMessage(value: unknown): value is WorkerMessage {
  if (!isRecord(value)) {
    return false;
  }
  if (value.type === 'error') {
    return typeof value.message === 'string';
  }
  return value.type === 'ready' && isReadyMessage(value);
}

export function isLoadMessage(value: unknown): value is LoadMessage {
  return isRecord(value) && value.type === 'load' && typeof value.url === 'string';
}
