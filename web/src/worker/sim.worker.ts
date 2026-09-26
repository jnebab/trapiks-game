import init, { Engine } from '../wasm/pkg/trapiks_sim_wasm.js';
import {
  isMainMessage,
  isMapMeta,
  type ErrorMessage,
  type LoadMessage,
  type MainMessage,
  type ReadyMessage,
  type Speed,
  type WorkerMessage,
} from '../sim/protocol';
import { EngineSession } from './engine-session';
import { fetchMap } from './fetch-map';
import { collectGeometry, transferList } from './geometry';
import { startWallTicks } from './loop';

const WALL_TICK_MS = 100;
const wasmReady = init();

let loading = false;
let session: EngineSession | undefined;
let pendingSpeed: Speed | undefined;

function post(message: WorkerMessage, transfer: Transferable[]): void {
  self.postMessage(message, { transfer });
}

function postError(error: unknown): void {
  const message: ErrorMessage = {
    type: 'error',
    message: error instanceof Error ? error.message : String(error),
  };
  self.postMessage(message);
}

function guarded(onTick: () => void): () => void {
  return () => {
    try {
      onTick();
    } catch (error) {
      postError(error);
    }
  };
}

async function load(request: LoadMessage): Promise<void> {
  const wasm = await wasmReady;
  const bytes = await fetchMap(request.url);
  const engine = Engine.load(bytes, request.config);
  const meta: unknown = engine.meta();
  if (!isMapMeta(meta)) {
    throw new Error('Invalid map meta from wasm');
  }
  const geometry = collectGeometry(engine);
  const ready: ReadyMessage = { type: 'ready', meta, ...geometry };
  post(ready, transferList(geometry));
  const started = new EngineSession(engine, wasm.memory, post);
  started.speed = pendingSpeed ?? started.speed;
  session = started;
  started.postSignals();
  startWallTicks(guarded(started.onTick), WALL_TICK_MS);
}

function startLoad(request: LoadMessage): void {
  if (loading) {
    return;
  }
  loading = true;
  load(request).catch(postError);
}

function setSpeed(speed: Speed): void {
  pendingSpeed = speed;
  if (session) {
    session.speed = speed;
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
      session?.returnBuffers(message.buffers);
  }
}

self.onmessage = (event: MessageEvent<unknown>) => {
  if (isMainMessage(event.data)) {
    route(event.data);
  }
};
