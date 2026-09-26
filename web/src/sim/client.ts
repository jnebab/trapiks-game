import {
  isWorkerMessage,
  type CommandResultsMessage,
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
import type { EditCommand } from '../generated/EditCommand';
import type { QuoteOutcome } from '../generated/QuoteOutcome';
import { snapshotTransfer } from './values';

export interface SimHandlers {
  onReady: (message: ReadyMessage) => void;
  onSnapshot: (message: SnapshotMessage) => void;
  onStats: (message: StatsMessage) => void;
  onSignals: (message: SignalsMessage) => void;
  onCommandResults: (message: CommandResultsMessage) => void;
  onError: (message: string) => void;
}

export interface SimClient {
  setSpeed: (speed: Speed) => void;
  returnBuffers: (buffers: SnapshotBuffers) => void;
  sendCommand: (command: EditCommand) => void;
  quote: (command: EditCommand) => Promise<QuoteOutcome>;
}

type QuoteWaiters = Map<number, (result: QuoteOutcome) => void>;

function dispatch(message: WorkerMessage, handlers: SimHandlers, quotes: QuoteWaiters): void {
  if (message.type === 'quoteResult') {
    quotes.get(message.id)?.(message.result);
    quotes.delete(message.id);
    return;
  }
  dispatchEvent(message, handlers);
}

function dispatchEvent(
  message: Exclude<WorkerMessage, { type: 'quoteResult' }>,
  handlers: SimHandlers,
): void {
  switch (message.type) {
    case 'commandResults':
      handlers.onCommandResults(message);
      return;
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
  const quotes: QuoteWaiters = new Map();
  worker.addEventListener('message', (event: MessageEvent<unknown>) => {
    if (!isWorkerMessage(event.data)) {
      handlers.onError('Malformed message from sim worker');
      return;
    }
    dispatch(event.data, handlers, quotes);
  });
  worker.addEventListener('error', (event: ErrorEvent) => {
    handlers.onError(event.message || 'Sim worker failed');
  });
  const send = (message: MainMessage, transfer: Transferable[] = []): void => {
    worker.postMessage(message, transfer);
  };
  const load: LoadMessage = { type: 'load', url, config };
  send(load);
  return clientFor(send, quotes);
}

type Send = (message: MainMessage, transfer?: Transferable[]) => void;

function clientFor(send: Send, quotes: QuoteWaiters): SimClient {
  let nextQuote = 0;
  return {
    setSpeed: (speed) => {
      send({ type: 'speed', speed });
    },
    returnBuffers: (buffers) => {
      send({ type: 'buffers', buffers }, snapshotTransfer(buffers));
    },
    sendCommand: (command) => {
      send({ type: 'command', command });
    },
    quote: (command) => {
      const id = nextQuote;
      nextQuote += 1;
      send({ type: 'quote', id, command });
      return new Promise((resolve) => {
        quotes.set(id, resolve);
      });
    },
  };
}
