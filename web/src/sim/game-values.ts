import type { Challenge } from '../generated/Challenge';
import type { EditCommand } from '../generated/EditCommand';
import type { FailReason } from '../generated/FailReason';
import type { RunResult } from '../generated/RunResult';
import type { RunState } from '../generated/RunState';
import type { Score } from '../generated/Score';
import { isEditCommand } from './edit-values';
import { isRecord } from './values';

export interface Region {
  x: number;
  y: number;
  radius: number;
}

export type GameMode = 'sandbox' | 'challenge';
export type RunPhase = 'baseline' | 'evaluate';

const FAIL_REASONS = new Set<FailReason>([
  'NotEnoughImprovement',
  'ThroughputDropped',
  'OverBudget',
]);

const CHALLENGE_NUMBERS = [
  'radius_m',
  'seed',
  'vehicles_per_hour',
  'budget',
  'target_improvement',
  'min_throughput_ratio',
] as const;

const RUN_NUMBERS = [
  'mean_delay_s',
  'throughput_per_hour',
  'created',
  'arrivals',
  'unserved',
  'stranded',
] as const;

function numbers(value: Record<string, unknown>, keys: readonly string[]): boolean {
  return keys.every((key) => typeof value[key] === 'number');
}

function isPair(value: unknown): boolean {
  return Array.isArray(value) && value.length === 2 && value.every((v) => typeof v === 'number');
}

function isCenter(value: unknown): boolean {
  if (!isRecord(value)) {
    return false;
  }
  if (isRecord(value.Local)) {
    return numbers(value.Local, ['x', 'y']);
  }
  return isRecord(value.LatLon) && numbers(value.LatLon, ['lat', 'lon']);
}

export function isChallenge(value: unknown): value is Challenge {
  if (!isRecord(value)) {
    return false;
  }
  const texts = ['id', 'name', 'blurb'].every((key) => typeof value[key] === 'string');
  return (
    texts && numbers(value, CHALLENGE_NUMBERS) && isCenter(value.center) && isPair(value.star_steps)
  );
}

export function isChallengeList(value: unknown): value is Challenge[] {
  return Array.isArray(value) && value.every(isChallenge);
}

export function isRunResult(value: unknown): value is RunResult {
  return isRecord(value) && numbers(value, RUN_NUMBERS);
}

export function isRunState(value: unknown): value is RunState {
  if (!isRecord(value)) {
    return false;
  }
  if (isRecord(value.Running)) {
    return numbers(value.Running, ['done_ticks', 'total_ticks']);
  }
  return isRunResult(value.Finished);
}

function isFailReason(value: unknown): value is FailReason {
  return [...FAIL_REASONS].some((reason) => reason === value);
}

export function isScore(value: unknown): value is Score {
  if (!isRecord(value)) {
    return false;
  }
  const reasonsOk = Array.isArray(value.reasons) && value.reasons.every(isFailReason);
  return (
    numbers(value, ['improvement', 'throughput_ratio', 'cost', 'stars']) &&
    typeof value.passed === 'boolean' &&
    reasonsOk
  );
}

export function isEditCommands(value: unknown): value is EditCommand[] {
  return Array.isArray(value) && value.every(isEditCommand);
}

export function isRegion(value: unknown): value is Region {
  return isRecord(value) && numbers(value, ['x', 'y', 'radius']);
}

export function isGameMode(value: unknown): value is GameMode {
  return value === 'sandbox' || value === 'challenge';
}

export function isRunPhase(value: unknown): value is RunPhase {
  return value === 'baseline' || value === 'evaluate';
}

export function isOptionalLog(value: unknown): boolean {
  return value === undefined || isEditCommands(value);
}

export function isPoint(value: unknown): value is { x: number; y: number } {
  return isRecord(value) && numbers(value, ['x', 'y']);
}
