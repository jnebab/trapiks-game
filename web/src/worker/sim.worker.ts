import init, { Engine } from '../wasm/pkg/trapiks_sim_wasm.js';
import { isLoadMessage, isMapMeta, type ErrorMessage, type ReadyMessage } from '../sim/protocol';
import { fetchMap } from './fetch-map';
import { collectGeometry, transferList } from './geometry';

const wasmReady = init();

async function load(url: string): Promise<void> {
  const bytes = await fetchMap(url);
  const engine = Engine.load(bytes);
  const meta: unknown = engine.meta();
  const geometry = collectGeometry(engine);
  engine.free();
  if (!isMapMeta(meta)) {
    throw new Error('Invalid map meta from wasm');
  }
  const message: ReadyMessage = { type: 'ready', meta, ...geometry };
  self.postMessage(message, { transfer: transferList(geometry) });
}

function postError(error: unknown): void {
  const message: ErrorMessage = {
    type: 'error',
    message: error instanceof Error ? error.message : String(error),
  };
  self.postMessage(message);
}

async function handleMessage(event: MessageEvent<unknown>): Promise<void> {
  try {
    await wasmReady;
    if (isLoadMessage(event.data)) {
      await load(event.data.url);
    }
  } catch (error) {
    postError(error);
  }
}

self.onmessage = (event: MessageEvent<unknown>) => {
  void handleMessage(event);
};
