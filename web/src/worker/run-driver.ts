import type { ChallengeRunner } from '../wasm/pkg/trapiks_sim_wasm.js';
import type { RunResult } from '../generated/RunResult';
import { isRunState } from '../sim/game-values';

const RUN_BUDGET_MS = 90;
const STEPS_PER_CALL = 32;

export type DriveOutcome =
  { kind: 'running'; done: number; total: number } | { kind: 'finished'; result: RunResult };

export function advance(runner: ChallengeRunner, steps = STEPS_PER_CALL): DriveOutcome {
  const state: unknown = runner.advance(steps);
  if (!isRunState(state)) {
    throw new Error('Invalid run state from wasm');
  }
  if ('Finished' in state) {
    return { kind: 'finished', result: state.Finished };
  }
  return { kind: 'running', done: state.Running.done_ticks, total: state.Running.total_ticks };
}

export function drive(runner: ChallengeRunner): DriveOutcome {
  const start = performance.now();
  let outcome = advance(runner);
  while (outcome.kind === 'running' && performance.now() - start < RUN_BUDGET_MS) {
    outcome = advance(runner);
  }
  return outcome;
}
