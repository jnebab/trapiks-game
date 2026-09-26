import type { BudgetState } from '../generated/BudgetState';
import type { CommandResult } from '../generated/CommandResult';
import type { EditCommand } from '../generated/EditCommand';
import type { QuoteOutcome } from '../generated/QuoteOutcome';
import { isRecord } from './values';

const COMMAND_KEYS = new Set([
  'DeleteRoad',
  'SetLanes',
  'SetSpeedLimit',
  'SetJunctionControl',
  'SetSignalTiming',
  'SetTurnAllowed',
  'BuildFlyover',
  'BuildRoundabout',
  'AddRoad',
  'SetDemand',
]);

function singleKey(value: Record<string, unknown>): string | undefined {
  const keys = Object.keys(value);
  return keys.length === 1 ? keys[0] : undefined;
}

export function isEditCommand(value: unknown): value is EditCommand {
  if (value === 'Undo') {
    return true;
  }
  if (!isRecord(value)) {
    return false;
  }
  const key = singleKey(value);
  return key !== undefined && COMMAND_KEYS.has(key) && isRecord(value[key]);
}

function isOkOrErr(value: unknown, isOk: (ok: unknown) => boolean): boolean {
  if (!isRecord(value)) {
    return false;
  }
  if (typeof value.Err === 'string') {
    return singleKey(value) === 'Err';
  }
  return singleKey(value) === 'Ok' && isOk(value.Ok);
}

export function isQuoteOutcome(value: unknown): value is QuoteOutcome {
  return isOkOrErr(value, (ok) => typeof ok === 'number');
}

function isEditOutcome(value: unknown): boolean {
  if (!isRecord(value)) {
    return false;
  }
  return (
    typeof value.cost === 'number' &&
    Array.isArray(value.changed_roads) &&
    Array.isArray(value.changed_nodes)
  );
}

export function isCommandResult(value: unknown): value is CommandResult {
  return (
    isRecord(value) && typeof value.seq === 'number' && isOkOrErr(value.outcome, isEditOutcome)
  );
}

export function isCommandResults(value: unknown): value is CommandResult[] {
  return Array.isArray(value) && value.every(isCommandResult);
}

export function isBudgetState(value: unknown): value is BudgetState {
  if (!isRecord(value)) {
    return false;
  }
  const limitOk = value.limit === null || typeof value.limit === 'number';
  return limitOk && typeof value.spent === 'number';
}
