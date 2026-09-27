import type { MapMeta } from '../generated/MapMeta';
import type { SimConfig } from '../generated/SimConfig';
import type { StatsSnapshot } from '../generated/StatsSnapshot';
import type { BudgetState } from '../generated/BudgetState';
import type { CommandResult } from '../generated/CommandResult';
import type { EditCommand } from '../generated/EditCommand';
import type { QuoteOutcome } from '../generated/QuoteOutcome';
import type { DeltaArrays } from './delta-arrays';
import type { AreaArrays, NodeArrays, RoadArrays } from './map-arrays';
import type { InspectionMessage, InspectMessage } from './inspect-values';
import type { SnapshotBuffers, Speed } from './values';
import type { GameMainMessage, GameWorkerMessage } from './protocol-game';
import type { GameMode, Region } from './game-values';
import type { ApproachMarkerArrays, JunctionShapeArrays, SignalPillArrays } from './street-arrays';

export type { SnapshotBuffers, Speed } from './values';
export type { ApproachMarkerArrays, JunctionShapeArrays, SignalPillArrays } from './street-arrays';
export type { DeltaArrays } from './delta-arrays';
export type {
  InspectTarget,
  Inspection,
  InspectionMessage,
  InspectMessage,
} from './inspect-values';
export type { AreaArrays, NodeArrays, RoadArrays } from './map-arrays';
export type {
  BaselineMessage,
  EvaluationMessage,
  NoticeMessage,
  RunProgressMessage,
  StartChallengeMessage,
  StartSandboxMessage,
} from './protocol-game';
export type { GameMode, Region, RunPhase } from './game-values';
export { isMapMeta } from './ready-values';

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
  | LoadMessage
  | SpeedMessage
  | BuffersMessage
  | CommandMessage
  | QuoteMessage
  | InspectMessage
  | GameMainMessage;

export interface ReadyMessage {
  type: 'ready';
  session: number;
  mode: GameMode;
  region: Region | null;
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
  session: number;
  message: string;
}

export interface SnapshotMessage {
  type: 'snapshot';
  session: number;
  tick: number;
  simTime: number;
  count: number;
  buffers: SnapshotBuffers;
}

export interface StatsMessage {
  type: 'stats';
  session: number;
  stats: StatsSnapshot;
  roadSpeedRatio: Float32Array<ArrayBuffer>;
  stepMs: Float64Array<ArrayBuffer>;
}

export interface SignalsMessage {
  type: 'signals';
  session: number;
  states: Uint8Array<ArrayBuffer>;
}

export interface QuoteResultMessage {
  type: 'quoteResult';
  session: number;
  id: number;
  result: QuoteOutcome;
}

export interface CommandResultsMessage {
  type: 'commandResults';
  session: number;
  results: CommandResult[];
  delta: DeltaArrays | null;
  budget: BudgetState;
  log: EditCommand[];
}

export type WorkerMessage =
  | ReadyMessage
  | ErrorMessage
  | SnapshotMessage
  | StatsMessage
  | SignalsMessage
  | QuoteResultMessage
  | InspectionMessage
  | CommandResultsMessage
  | GameWorkerMessage;

export { isMainMessage, isWorkerMessage } from './protocol-guards';
