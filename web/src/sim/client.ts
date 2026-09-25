import type { EngineInfo } from '../generated/EngineInfo';
import { isWorkerMessage } from './protocol';

export function startSim(onReady: (info: EngineInfo) => void): Worker {
  const worker = new Worker(new URL('../worker/sim.worker.ts', import.meta.url), {
    type: 'module',
  });
  worker.addEventListener('message', (event: MessageEvent<unknown>) => {
    if (!isWorkerMessage(event.data)) {
      return;
    }
    onReady(event.data.info);
  });
  return worker;
}
