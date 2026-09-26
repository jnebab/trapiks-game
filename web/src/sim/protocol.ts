import type { MapMeta } from '../generated/MapMeta';
import type { SimConfig } from '../generated/SimConfig';
import type { StatsSnapshot } from '../generated/StatsSnapshot';
import type { BudgetState } from '../generated/BudgetState';
import type { CommandResult } from '../generated/CommandResult';
import type { EditCommand } from '../generated/EditCommand';
import type { QuoteOutcome } from '../generated/QuoteOutcome';
import { isDeltaArrays, type DeltaArrays } from './delta-arrays';
import {
  areaShape,
  nodeShape,
  roadShape,
  type AreaArrays,
  type NodeArrays,
  type RoadArrays,
} from './map-arrays';
import { isBudgetState, isCommandResults, isEditCommand, isQuoteOutcome } from './edit-values';
import {
  hasArrays,
  isRecord,
  isSimConfig,
  isSnapshotBuffers,
  isSpeed,
  isStatsSnapshot,
  type SnapshotBuffers,
  type Speed,
} from './values';
import {
  junctionShape,
  markerShape,
  signalPillShape,
  type ApproachMarkerArrays,
  type JunctionShapeArrays,
  type SignalPillArrays,
} from './street-arrays';

export type { SnapshotBuffers, Speed } from './values';
export type { ApproachMarkerArrays, JunctionShapeArrays, SignalPillArrays } from './street-arrays';
export type { DeltaArrays } from './delta-arrays';
export type { AreaArrays, NodeArrays, RoadArrays } from './map-arrays';

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

export interface CommandMessage {
  type: 'command';
  command: EditCommand;
}

export interface QuoteMessage {
  type: 'quote';
  id: number;
  command: EditCommand;
}

export type MainMessage =
  LoadMessage | SpeedMessage | BuffersMessage | CommandMessage | QuoteMessage;

export interface ReadyMessage {
  type: 'ready';
  meta: MapMeta;
  roads: RoadArrays;
  nodes: NodeArrays;
  areas: AreaArrays;
  roadSetbacks: Float32Array;
  junctions: JunctionShapeArrays;
  markers: ApproachMarkerArrays;
  signalPills: SignalPillArrays;
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

export interface SignalsMessage {
  type: 'signals';
  states: Uint8Array<ArrayBuffer>;
}

export interface QuoteResultMessage {
  type: 'quoteResult';
  id: number;
  result: QuoteOutcome;
}

export interface CommandResultsMessage {
  type: 'commandResults';
  results: CommandResult[];
  delta: DeltaArrays | null;
  budget: BudgetState;
}

export type WorkerMessage =
  | ReadyMessage
  | ErrorMessage
  | SnapshotMessage
  | StatsMessage
  | SignalsMessage
  | QuoteResultMessage
  | CommandResultsMessage;

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

function isReadyMessage(value: Record<string, unknown>): boolean {
  return (
    isMapMeta(value.meta) &&
    hasArrays(value.roads, roadShape) &&
    hasArrays(value.nodes, nodeShape) &&
    hasArrays(value.areas, areaShape) &&
    value.roadSetbacks instanceof Float32Array &&
    hasArrays(value.junctions, junctionShape) &&
    hasArrays(value.markers, markerShape) &&
    hasArrays(value.signalPills, signalPillShape)
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

function isCommandResultsMessage(value: Record<string, unknown>): boolean {
  const deltaOk = value.delta === null || isDeltaArrays(value.delta);
  return isCommandResults(value.results) && deltaOk && isBudgetState(value.budget);
}

const workerGuards: Record<WorkerMessage['type'], (value: Record<string, unknown>) => boolean> = {
  ready: isReadyMessage,
  error: (value) => typeof value.message === 'string',
  snapshot: isSnapshotMessage,
  stats: isStatsMessage,
  signals: (value) => value.states instanceof Uint8Array,
  quoteResult: (value) => typeof value.id === 'number' && isQuoteOutcome(value.result),
  commandResults: isCommandResultsMessage,
};

const mainGuards: Record<MainMessage['type'], (value: Record<string, unknown>) => boolean> = {
  load: (value) => typeof value.url === 'string' && isSimConfig(value.config),
  speed: (value) => isSpeed(value.speed),
  buffers: (value) => isSnapshotBuffers(value.buffers),
  command: (value) => isEditCommand(value.command),
  quote: (value) => typeof value.id === 'number' && isEditCommand(value.command),
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
