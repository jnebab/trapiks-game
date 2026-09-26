import {
  isWorkerMessage,
  type LoadMessage,
  type MainMessage,
  type ReadyMessage,
  type SignalsMessage,
  type SnapshotBuffers,
  type SnapshotMessage,
  type Speed,
  type StatsMessage,
  type WorkerMessage,
} from './protocol';
import type { SimConfig } from '../generated/SimConfig';
import { snapshotTransfer } from './values';

export interface SimHandlers {
  onReady: (message: ReadyMessage) => void;
  onSnapshot: (message: SnapshotMessage) => void;
  onStats: (message: StatsMessage) => void;
  onSignals: (message: SignalsMessage) => void;
  onError: (message: string) => void;
}

export interface SimClient {
  setSpeed: (speed: Speed) => void;
  returnBuffers: (buffers: SnapshotBuffers) => void;
}

function dispatch(message: WorkerMessage, handlers: SimHandlers): void {
  switch (message.type) {
    case 'ready':
      handlers.onReady(message);
      return;
    case 'snapshot':
      handlers.onSnapshot(message);
      return;
    case 'stats':
      handlers.onStats(message);
      return;
    case 'signals':
      handlers.onSignals(message);
      return;
    case 'error':
      handlers.onError(message.message);
  }
}

export function startSim(url: string, config: SimConfig, handlers: SimHandlers): SimClient {
  const worker = new Worker(new URL('../worker/sim.worker.ts', import.meta.url), {
    type: 'module',
  });
  worker.addEventListener('message', (event: MessageEvent<unknown>) => {
    if (!isWorkerMessage(event.data)) {
      handlers.onError('Malformed message from sim worker');
      return;
    }
    dispatch(event.data, handlers);
  });
  worker.addEventListener('error', (event: ErrorEvent) => {
    handlers.onError(event.message || 'Sim worker failed');
  });
  const send = (message: MainMessage, transfer: Transferable[] = []): void => {
    worker.postMessage(message, transfer);
  };
  const load: LoadMessage = { type: 'load', url, config };
  send(load);
  return {
    setSpeed: (speed) => {
      send({ type: 'speed', speed });
    },
    returnBuffers: (buffers) => {
      send({ type: 'buffers', buffers }, snapshotTransfer(buffers));
    },
  };
}
