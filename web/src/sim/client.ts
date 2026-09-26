import {
  isWorkerMessage,
  type CommandResultsMessage,
  type InspectTarget,
  type Inspection,
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
  inspect: (target: InspectTarget) => Promise<Inspection | null>;
}

type Waiters<T> = Map<number, (result: T) => void>;

interface Pending {
  quotes: Waiters<QuoteOutcome>;
  inspections: Waiters<Inspection | null>;
}

function settle<T>(waiters: Waiters<T>, id: number, result: T): void {
  waiters.get(id)?.(result);
  waiters.delete(id);
}

function dispatch(message: WorkerMessage, handlers: SimHandlers, pending: Pending): void {
  if (message.type === 'quoteResult') {
    settle(pending.quotes, message.id, message.result);
    return;
  }
  if (message.type === 'inspection') {
    settle(pending.inspections, message.id, message.target);
    return;
  }
  dispatchEvent(message, handlers);
}

function dispatchEvent(
  message: Exclude<WorkerMessage, { type: 'quoteResult' | 'inspection' }>,
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
  const pending: Pending = { quotes: new Map(), inspections: new Map() };
  worker.addEventListener('message', (event: MessageEvent<unknown>) => {
    if (!isWorkerMessage(event.data)) {
      handlers.onError('Malformed message from sim worker');
      return;
    }
    dispatch(event.data, handlers, pending);
  });
  worker.addEventListener('error', (event: ErrorEvent) => {
    handlers.onError(event.message || 'Sim worker failed');
  });
  const send = (message: MainMessage, transfer: Transferable[] = []): void => {
    worker.postMessage(message, transfer);
  };
  const load: LoadMessage = { type: 'load', url, config };
  send(load);
  return clientFor(send, pending);
}

type Send = (message: MainMessage, transfer?: Transferable[]) => void;

function request<T>(waiters: Waiters<T>, id: number): Promise<T> {
  return new Promise((resolve) => {
    waiters.set(id, resolve);
  });
}

function clientFor(send: Send, pending: Pending): SimClient {
  let nextId = 0;
  const takeId = (): number => {
    nextId += 1;
    return nextId;
  };
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
      const id = takeId();
      send({ type: 'quote', id, command });
      return request(pending.quotes, id);
    },
    inspect: (target) => {
      const id = takeId();
      send({ type: 'inspect', id, target });
      return request(pending.inspections, id);
    },
  };
}
