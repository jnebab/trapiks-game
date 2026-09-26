import { ChallengeRunner, score, type MapHandle } from '../wasm/pkg/trapiks_sim_wasm.js';
import type { Challenge } from '../generated/Challenge';
import type { RunResult } from '../generated/RunResult';
import type { Score } from '../generated/Score';
import { isScore } from '../sim/game-values';
import type { RunPhase } from '../sim/protocol';
import type { EngineSession, Post } from './engine-session';
import { advance, drive } from './run-driver';

interface ActiveRun {
  phase: RunPhase;
  runner: ChallengeRunner;
}

export interface ChallengeSetup {
  map: MapHandle;
  challenge: Challenge;
  post: Post;
}

export class ChallengeSession {
  private baseline: RunResult | undefined;
  private run: ActiveRun | undefined;

  constructor(
    private readonly setup: ChallengeSetup,
    readonly live: EngineSession,
    baselineRunner: ChallengeRunner,
  ) {
    this.run = { phase: 'baseline', runner: baselineRunner };
  }

  static startBaseline(setup: ChallengeSetup, session: number): ChallengeRunner {
    const runner = new ChallengeRunner(setup.map, setup.challenge, []);
    announceStart(setup.post, session, 'baseline', runner);
    return runner;
  }

  get session(): number {
    return this.live.session;
  }

  onTick(): void {
    const run = this.run;
    this.live.onTick(run !== undefined);
    if (run !== undefined) {
      this.step(run);
    }
  }

  evaluate(): void {
    if (this.baseline === undefined || this.run !== undefined) {
      return;
    }
    const runner = new ChallengeRunner(
      this.setup.map,
      this.setup.challenge,
      this.live.commandLog(),
    );
    this.run = { phase: 'evaluate', runner };
    announceStart(this.setup.post, this.session, 'evaluate', runner);
  }

  dispose(): void {
    this.run?.runner.free();
    this.run = undefined;
    this.live.dispose();
  }

  private step(run: ActiveRun): void {
    const outcome = drive(run.runner);
    if (outcome.kind === 'running') {
      this.postProgress(run.phase, outcome.done, outcome.total);
      return;
    }
    run.runner.free();
    this.run = undefined;
    this.finish(run.phase, outcome.result);
  }

  private finish(phase: RunPhase, result: RunResult): void {
    const session = this.session;
    if (phase === 'baseline') {
      this.baseline = result;
      this.setup.post({ type: 'baseline', session, result }, []);
      return;
    }
    this.setup.post({ type: 'evaluation', session, result, score: this.score(result) }, []);
  }

  private score(after: RunResult): Score {
    return scoreOf(
      this.setup.challenge,
      this.baseline ?? after,
      after,
      this.live.budgetState().spent,
    );
  }

  private postProgress(phase: RunPhase, done: number, total: number): void {
    this.setup.post({ type: 'runProgress', session: this.session, phase, done, total }, []);
  }
}

function announceStart(post: Post, session: number, phase: RunPhase, runner: ChallengeRunner) {
  const outcome = advance(runner, 0);
  const total = outcome.kind === 'running' ? outcome.total : 0;
  post({ type: 'runProgress', session, phase, done: 0, total }, []);
}

function scoreOf(challenge: Challenge, baseline: RunResult, after: RunResult, cost: number): Score {
  const result: unknown = score(challenge, baseline, after, cost);
  if (!isScore(result)) {
    throw new Error('Invalid score from wasm');
  }
  return result;
}
