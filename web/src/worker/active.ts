import type { MapHandle } from '../wasm/pkg/trapiks_sim_wasm.js';
import type { Challenge } from '../generated/Challenge';
import type { EditCommand } from '../generated/EditCommand';
import type { SimConfig } from '../generated/SimConfig';
import type { Region, StartChallengeMessage, StartSandboxMessage } from '../sim/protocol';
import { isPoint } from '../sim/game-values';
import { ChallengeSession } from './challenge-session';
import type { EngineSession } from './engine-session';
import { startEngine, type WorkerContext } from './start-engine';

const SANDBOX_SEED = 1;

export interface ActiveSession {
  readonly session: number;
  readonly live: EngineSession;
  readonly isSandbox: boolean;
  onTick: () => void;
  evaluate: () => void;
  dispose: () => void;
}

export function sandboxConfig(vehiclesPerHour: number): SimConfig {
  return { seed: SANDBOX_SEED, mode: 'City', vehicles_per_hour: vehiclesPerHour };
}

function wrapSandbox(live: EngineSession): ActiveSession {
  return {
    session: live.session,
    live,
    isSandbox: true,
    onTick: () => {
      live.onTick();
    },
    evaluate: () => undefined,
    dispose: () => {
      live.dispose();
    },
  };
}

export function openInitial(ctx: WorkerContext, config: SimConfig): ActiveSession {
  const request = { session: 0, config, mode: 'sandbox', region: null, log: undefined } as const;
  return wrapSandbox(startEngine(ctx, request));
}

export function openSandbox(ctx: WorkerContext, message: StartSandboxMessage): ActiveSession {
  const live = startEngine(ctx, {
    session: message.session,
    config: sandboxConfig(message.vehiclesPerHour),
    mode: 'sandbox',
    region: null,
    log: message.log,
  });
  return wrapSandbox(live);
}

function regionOf(map: MapHandle, challenge: Challenge): Region {
  const center: unknown = map.challengeCenter(challenge);
  if (!isPoint(center)) {
    throw new Error('Invalid challenge centre from wasm');
  }
  return { x: center.x, y: center.y, radius: challenge.radius_m };
}

function challengeConfig(challenge: Challenge, region: Region): SimConfig {
  return {
    seed: challenge.seed,
    mode: { Region: { center_x: region.x, center_y: region.y, radius: region.radius } },
    vehicles_per_hour: challenge.vehicles_per_hour,
    budget: challenge.budget,
  };
}

function wrapChallenge(challenge: ChallengeSession): ActiveSession {
  return {
    session: challenge.session,
    live: challenge.live,
    isSandbox: false,
    onTick: () => {
      challenge.onTick();
    },
    evaluate: () => {
      challenge.evaluate();
    },
    dispose: () => {
      challenge.dispose();
    },
  };
}

export function openChallenge(ctx: WorkerContext, message: StartChallengeMessage): ActiveSession {
  const { challenge, session } = message;
  const setup = { map: ctx.map, challenge, post: ctx.post };
  const baseline = ChallengeSession.startBaseline(setup, session);
  const region = regionOf(ctx.map, challenge);
  const log: EditCommand[] | undefined = message.log;
  const live = startEngine(ctx, {
    session,
    config: challengeConfig(challenge, region),
    mode: 'challenge',
    region,
    log,
  });
  return wrapChallenge(new ChallengeSession(setup, live, baseline));
}
