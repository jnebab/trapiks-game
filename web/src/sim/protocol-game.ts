import type { Challenge } from '../generated/Challenge';
import type { EditCommand } from '../generated/EditCommand';
import type { RunResult } from '../generated/RunResult';
import type { Score } from '../generated/Score';
import {
  isChallenge,
  isOptionalLog,
  isRunPhase,
  isRunResult,
  isScore,
  type RunPhase,
} from './game-values';
import { isRecord } from './values';

type Guard = (value: Record<string, unknown>) => boolean;

export interface StartSandboxMessage {
  type: 'startSandbox';
  session: number;
  vehiclesPerHour: number;
  log?: EditCommand[];
}

export interface StartChallengeMessage {
  type: 'startChallenge';
  session: number;
  challenge: Challenge;
  log?: EditCommand[];
}

export interface EvaluateMessage {
  type: 'evaluate';
}

export interface SetDemandMessage {
  type: 'setDemand';
  vehiclesPerHour: number;
}

export type GameMainMessage =
  StartSandboxMessage | StartChallengeMessage | EvaluateMessage | SetDemandMessage;

export interface RunProgressMessage {
  type: 'runProgress';
  session: number;
  phase: RunPhase;
  done: number;
  total: number;
}

export interface BaselineMessage {
  type: 'baseline';
  session: number;
  result: RunResult;
}

export interface EvaluationMessage {
  type: 'evaluation';
  session: number;
  result: RunResult;
  score: Score;
}

export interface NoticeMessage {
  type: 'notice';
  session: number;
  text: string;
}

export type GameWorkerMessage =
  RunProgressMessage | BaselineMessage | EvaluationMessage | NoticeMessage;

function isCount(value: unknown): boolean {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0;
}

export const gameMainGuards: Record<GameMainMessage['type'], Guard> = {
  startSandbox: (value) =>
    isCount(value.session) && isCount(value.vehiclesPerHour) && isOptionalLog(value.log),
  startChallenge: (value) =>
    isCount(value.session) && isChallenge(value.challenge) && isOptionalLog(value.log),
  evaluate: () => true,
  setDemand: (value) => isCount(value.vehiclesPerHour),
};

export const gameWorkerGuards: Record<GameWorkerMessage['type'], Guard> = {
  runProgress: (value) => isRunPhase(value.phase) && isCount(value.done) && isCount(value.total),
  baseline: (value) => isRunResult(value.result),
  evaluation: (value) => isRunResult(value.result) && isScore(value.score),
  notice: (value) => typeof value.text === 'string',
};

export function hasSession(value: unknown): boolean {
  return isRecord(value) && typeof value.session === 'number';
}
