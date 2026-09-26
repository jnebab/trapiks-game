import init, { MapHandle } from '../wasm/pkg/trapiks_sim_wasm.js';
import {
  isMainMessage,
  type ErrorMessage,
  type LoadMessage,
  type MainMessage,
  type Speed,
  type StartChallengeMessage,
  type StartSandboxMessage,
  type WorkerMessage,
} from '../sim/protocol';
import { openChallenge, openInitial, openSandbox, type ActiveSession } from './active';
import { fetchMap } from './fetch-map';
import { startWallTicks } from './loop';
import type { WorkerContext } from './start-engine';

const WALL_TICK_MS = 100;
const wasmReady = init();

type StartMessage = StartSandboxMessage | StartChallengeMessage;

let loading = false;
let context: WorkerContext | undefined;
let active: ActiveSession | undefined;
let pendingStart: StartMessage | undefined;
let speed: Speed = 1;

function post(message: WorkerMessage, transfer: Transferable[]): void {
  self.postMessage(message, { transfer });
}

function postError(error: unknown): void {
  const message: ErrorMessage = {
    type: 'error',
    session: active?.session ?? 0,
    message: error instanceof Error ? error.message : String(error),
  };
  self.postMessage(message);
}

function guarded(action: () => void): void {
  try {
    action();
  } catch (error) {
    postError(error);
  }
}

function replace(open: () => ActiveSession): void {
  active?.dispose();
  active = undefined;
  const next = open();
  next.live.speed = speed;
  active = next;
}

function start(ctx: WorkerContext, message: StartMessage): void {
  if (message.type === 'startSandbox') {
    replace(() => openSandbox(ctx, message));
    return;
  }
  replace(() => openChallenge(ctx, message));
}

async function load(request: LoadMessage): Promise<void> {
  const wasm = await wasmReady;
  const bytes = await fetchMap(request.url);
  const ctx: WorkerContext = { map: new MapHandle(bytes), memory: wasm.memory, post };
  context = ctx;
  const queued = pendingStart;
  pendingStart = undefined;
  guarded(() => {
    if (queued === undefined) {
      replace(() => openInitial(ctx, request.config));
    } else {
      start(ctx, queued);
    }
  });
  startWallTicks(() => {
    guarded(() => active?.onTick());
  }, WALL_TICK_MS);
}

function startLoad(request: LoadMessage): void {
  if (loading) {
    return;
  }
  loading = true;
  load(request).catch(postError);
}

function requestStart(message: StartMessage): void {
  if (context === undefined) {
    pendingStart = message;
    return;
  }
  const ctx = context;
  guarded(() => {
    start(ctx, message);
  });
}

function setSpeed(next: Speed): void {
  speed = next;
  if (active) {
    active.live.speed = next;
  }
}

function withSession(action: (current: ActiveSession) => void): void {
  if (active === undefined) {
    postError(new Error('Sim is not loaded'));
    return;
  }
  const current = active;
  guarded(() => {
    action(current);
  });
}

function routeGame(message: MainMessage): void {
  switch (message.type) {
    case 'startSandbox':
    case 'startChallenge':
      requestStart(message);
      return;
    case 'evaluate':
      withSession((current) => {
        current.evaluate();
      });
      return;
    case 'setDemand':
      withSession((current) => {
        if (current.isSandbox) {
          current.live.enqueue({ SetDemand: { vehicles_per_hour: message.vehiclesPerHour } });
        }
      });
  }
}

function route(message: MainMessage): void {
  switch (message.type) {
    case 'load':
      startLoad(message);
      return;
    case 'speed':
      setSpeed(message.speed);
      return;
    case 'buffers':
      active?.live.returnBuffers(message.buffers);
      return;
    case 'command':
      withSession((current) => {
        current.live.enqueue(message.command);
      });
      return;
    case 'quote':
      withSession((current) => {
        current.live.quote(message.id, message.command);
      });
      return;
    case 'inspect':
      withSession((current) => {
        current.live.inspect(message.id, message.target);
      });
      return;
    default:
      routeGame(message);
  }
}

self.onmessage = (event: MessageEvent<unknown>) => {
  if (isMainMessage(event.data)) {
    route(event.data);
  }
};
