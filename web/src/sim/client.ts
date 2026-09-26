import {
  isWorkerMessage,
  type BaselineMessage,
  type CommandResultsMessage,
  type EvaluationMessage,
  type InspectTarget,
  type Inspection,
  type LoadMessage,
  type MainMessage,
  type ReadyMessage,
  type RunProgressMessage,
  type SignalsMessage,
  type SnapshotBuffers,
  type SnapshotMessage,
  type Speed,
  type StatsMessage,
  type WorkerMessage,
} from './protocol';
import type { Challenge } from '../generated/Challenge';
import type { SimConfig } from '../generated/SimConfig';
import type { EditCommand } from '../generated/EditCommand';
import type { QuoteOutcome } from '../generated/QuoteOutcome';
import { snapshotTransfer } from './values';
import { createPending, type Pending } from './pending';

export interface SimHandlers {
  onReady: (message: ReadyMessage) => void;
  onSnapshot: (message: SnapshotMessage) => void;
  onStats: (message: StatsMessage) => void;
  onSignals: (message: SignalsMessage) => void;
  onCommandResults: (message: CommandResultsMessage) => void;
  onRunProgress: (message: RunProgressMessage) => void;
  onBaseline: (message: BaselineMessage) => void;
  onEvaluation: (message: EvaluationMessage) => void;
  onNotice: (text: string) => void;
  onError: (message: string) => void;
}

export interface SimClient {
  readonly session: number;
  setSpeed: (speed: Speed) => void;
  returnBuffers: (buffers: SnapshotBuffers) => void;
  sendCommand: (command: EditCommand) => void;
  quote: (command: EditCommand) => Promise<QuoteOutcome>;
  inspect: (target: InspectTarget) => Promise<Inspection | null>;
  startSandbox: (vehiclesPerHour: number, log?: EditCommand[]) => void;
  startChallenge: (challenge: Challenge, log?: EditCommand[]) => void;
  evaluate: () => void;
  setDemand: (vehiclesPerHour: number) => void;
}

type Send = (message: MainMessage, transfer?: Transferable[]) => void;

function dispatch(message: WorkerMessage, handlers: SimHandlers, pending: Pending): void {
  if (message.type === 'quoteResult') {
    pending.quotes.settle(message.id, message.result);
    return;
  }
  if (message.type === 'inspection') {
    pending.inspections.settle(message.id, message.target);
    return;
  }
  dispatchEvent(message, handlers);
}

type EventMessage = Exclude<WorkerMessage, { type: 'quoteResult' | 'inspection' }>;

function dispatchGame(message: EventMessage, handlers: SimHandlers): void {
  switch (message.type) {
    case 'runProgress':
      handlers.onRunProgress(message);
      return;
    case 'baseline':
      handlers.onBaseline(message);
      return;
    case 'evaluation':
      handlers.onEvaluation(message);
      return;
    case 'notice':
      handlers.onNotice(message.text);
      return;
    case 'error':
      handlers.onError(message.message);
  }
}

function dispatchEvent(message: EventMessage, handlers: SimHandlers): void {
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
    default:
      dispatchGame(message, handlers);
  }
}

interface ClientState {
  session: number;
}

function receive(data: unknown, state: ClientState, send: Send, route: (m: WorkerMessage) => void) {
  if (!isWorkerMessage(data)) {
    return false;
  }
  if (data.session === state.session) {
    route(data);
    return true;
  }
  if (data.type === 'snapshot') {
    send({ type: 'buffers', buffers: data.buffers }, snapshotTransfer(data.buffers));
  }
  return true;
}

export function startSim(url: string, config: SimConfig, handlers: SimHandlers): SimClient {
  const worker = new Worker(new URL('../worker/sim.worker.ts', import.meta.url), {
    type: 'module',
  });
  const pending = createPending();
  const state: ClientState = { session: 0 };
  const send: Send = (message, transfer = []) => {
    worker.postMessage(message, transfer);
  };
  worker.addEventListener('message', (event: MessageEvent<unknown>) => {
    const route = (message: WorkerMessage): void => {
      dispatch(message, handlers, pending);
    };
    if (!receive(event.data, state, send, route)) {
      handlers.onError('Malformed message from sim worker');
    }
  });
  worker.addEventListener('error', (event: ErrorEvent) => {
    handlers.onError(event.message || 'Sim worker failed');
  });
  const load: LoadMessage = { type: 'load', url, config };
  send(load);
  return clientFor(send, pending, state);
}

function nextSession(state: ClientState, pending: Pending): number {
  state.session += 1;
  pending.rejectAll();
  return state.session;
}

function clientFor(send: Send, pending: Pending, state: ClientState): SimClient {
  return {
    get session() {
      return state.session;
    },
    setSpeed: (speed) => {
      send({ type: 'speed', speed });
    },
    returnBuffers: (buffers) => {
      send({ type: 'buffers', buffers }, snapshotTransfer(buffers));
    },
    sendCommand: (command) => {
      send({ type: 'command', command });
    },
    quote: (command) =>
      pending.quotes.request((id) => {
        send({ type: 'quote', id, command });
      }),
    inspect: (target) =>
      pending.inspections.request((id) => {
        send({ type: 'inspect', id, target });
      }),
    startSandbox: (vehiclesPerHour, log) => {
      const session = nextSession(state, pending);
      send({ type: 'startSandbox', session, vehiclesPerHour, ...(log ? { log } : {}) });
    },
    startChallenge: (challenge, log) => {
      const session = nextSession(state, pending);
      send({ type: 'startChallenge', session, challenge, ...(log ? { log } : {}) });
    },
    evaluate: () => {
      send({ type: 'evaluate' });
    },
    setDemand: (vehiclesPerHour) => {
      send({ type: 'setDemand', vehiclesPerHour });
    },
  };
}
