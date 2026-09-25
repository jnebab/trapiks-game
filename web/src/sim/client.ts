import type { ReadyMessage } from './protocol';
import { isWorkerMessage, type LoadMessage } from './protocol';

export interface SimHandlers {
  onReady: (message: ReadyMessage) => void;
  onError: (message: string) => void;
}

export function startSim(url: string, handlers: SimHandlers): Worker {
  const worker = new Worker(new URL('../worker/sim.worker.ts', import.meta.url), {
    type: 'module',
  });
  worker.addEventListener('message', (event: MessageEvent<unknown>) => {
    const data = event.data;
    if (!isWorkerMessage(data)) {
      handlers.onError('Malformed message from sim worker');
      return;
    }
    if (data.type === 'error') {
      handlers.onError(data.message);
      return;
    }
    handlers.onReady(data);
  });
  worker.addEventListener('error', (event: ErrorEvent) => {
    handlers.onError(event.message || 'Sim worker failed');
  });
  const load: LoadMessage = { type: 'load', url };
  worker.postMessage(load);
  return worker;
}
