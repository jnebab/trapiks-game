import init, { engineInfo } from '../wasm/pkg/trapiks_sim_wasm.js';
import { isEngineInfo, type ReadyMessage } from '../sim/protocol';

await init();

const value: unknown = engineInfo();
if (!isEngineInfo(value)) {
  throw new Error('Invalid engine info from wasm');
}

const message: ReadyMessage = { type: 'ready', info: value };
self.postMessage(message);
