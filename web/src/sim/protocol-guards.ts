import { isDeltaArrays } from './delta-arrays';
import { isBudgetState, isCommandResults, isEditCommand, isQuoteOutcome } from './edit-values';
import { isEditCommands } from './game-values';
import { isInspectionMessage, isInspectMessage } from './inspect-values';
import { gameMainGuards, gameWorkerGuards, hasSession } from './protocol-game';
import type { MainMessage, WorkerMessage } from './protocol';
import { isReadyMessage } from './ready-values';
import { isRecord, isSimConfig, isSnapshotBuffers, isSpeed, isStatsSnapshot } from './values';

function isSnapshotMessage(value: Record<string, unknown>): boolean {
  return (
    typeof value.tick === 'number' &&
    typeof value.simTime === 'number' &&
    typeof value.count === 'number' &&
    isSnapshotBuffers(value.buffers)
  );
}

function isStatsMessage(value: Record<string, unknown>): boolean {
  return (
    isStatsSnapshot(value.stats) &&
    value.roadSpeedRatio instanceof Float32Array &&
    value.stepMs instanceof Float64Array
  );
}

function isCommandResultsMessage(value: Record<string, unknown>): boolean {
  const deltaOk = value.delta === null || isDeltaArrays(value.delta);
  return (
    isCommandResults(value.results) &&
    deltaOk &&
    isBudgetState(value.budget) &&
    isEditCommands(value.log)
  );
}

const workerGuards: Record<WorkerMessage['type'], (value: Record<string, unknown>) => boolean> = {
  ready: isReadyMessage,
  error: (value) => typeof value.message === 'string',
  snapshot: isSnapshotMessage,
  stats: isStatsMessage,
  signals: (value) => value.states instanceof Uint8Array,
  quoteResult: (value) => typeof value.id === 'number' && isQuoteOutcome(value.result),
  inspection: isInspectionMessage,
  commandResults: isCommandResultsMessage,
  ...gameWorkerGuards,
};

const mainGuards: Record<MainMessage['type'], (value: Record<string, unknown>) => boolean> = {
  load: (value) => typeof value.url === 'string' && isSimConfig(value.config),
  speed: (value) => isSpeed(value.speed),
  buffers: (value) => isSnapshotBuffers(value.buffers),
  command: (value) => isEditCommand(value.command),
  quote: (value) => typeof value.id === 'number' && isEditCommand(value.command),
  inspect: isInspectMessage,
  ...gameMainGuards,
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
  return hasSession(value) && matches(value, workerGuards);
}

export function isMainMessage(value: unknown): value is MainMessage {
  return matches(value, mainGuards);
}
