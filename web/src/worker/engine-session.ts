import type { Engine } from '../wasm/pkg/trapiks_sim_wasm.js';
import type {
  CommandResultsMessage,
  SignalsMessage,
  SnapshotMessage,
  Speed,
  StatsMessage,
  WorkerMessage,
} from '../sim/protocol';
import { statesChanged } from './signal-diff';
import { isStatsSnapshot, snapshotTransfer, type SnapshotBuffers } from '../sim/values';
import { BufferPool } from './buffer-pool';
import { MemoryViews } from './memory-views';
import { deltaTransfer } from '../sim/delta-arrays';
import { isBudgetState, isCommandResults, isQuoteOutcome } from '../sim/edit-values';
import type { EditCommand } from '../generated/EditCommand';
import { changedIds, collectDelta } from './delta';

const STEP_BUDGET_MS = 90;
const STATS_EVERY = 10;
const DT = 0.1;

export type Post = (message: WorkerMessage, transfer: Transferable[]) => void;

function snapshotLayout(engine: Engine): ConstructorParameters<typeof MemoryViews>[1] {
  const p = engine.snapshotPointers();
  const layout = {
    ids: p.ids,
    x: p.x,
    y: p.y,
    heading: p.heading,
    style: p.style,
    capacity: p.capacity,
  };
  p.free();
  return layout;
}

export class EngineSession {
  speed: Speed = 1;
  private readonly views: MemoryViews;
  private readonly pool = new BufferPool();
  private wallTicks = 0;
  private lastSignals: Uint8Array | undefined;

  constructor(
    private readonly engine: Engine,
    memory: WebAssembly.Memory,
    private readonly post: Post,
  ) {
    this.views = new MemoryViews(memory, snapshotLayout(engine));
  }

  readonly onTick = (): void => {
    if (this.runSteps() === 0) {
      this.engine.flushCommands();
    }
    this.postCommandResults();
    this.postSnapshot();
    this.postSignals();
    this.wallTicks += 1;
    if (this.wallTicks % STATS_EVERY === 0) {
      this.postStats();
    }
  };

  postSignals(): void {
    const states = new Uint8Array(this.engine.signalStates());
    if (!statesChanged(this.lastSignals, states)) {
      return;
    }
    this.lastSignals = states.slice();
    const message: SignalsMessage = { type: 'signals', states };
    this.post(message, [states.buffer]);
  }

  returnBuffers(buffers: SnapshotBuffers): void {
    this.pool.give(buffers);
  }

  enqueue(command: EditCommand): void {
    this.engine.enqueue(command);
  }

  quote(id: number, command: EditCommand): void {
    const result: unknown = this.engine.quote(command);
    if (!isQuoteOutcome(result)) {
      throw new Error('Invalid quote from wasm');
    }
    this.post({ type: 'quoteResult', id, result }, []);
  }

  private runSteps(): number {
    if (this.speed === 0) {
      return 0;
    }
    const limit = this.speed === 'max' ? Infinity : this.speed;
    const start = performance.now();
    let steps = 0;
    while (steps < limit) {
      this.engine.step();
      steps += 1;
      if (performance.now() - start >= STEP_BUDGET_MS) {
        break;
      }
    }
    return steps;
  }

  private postCommandResults(): void {
    const results: unknown = this.engine.takeResults();
    const budget: unknown = this.engine.budget();
    if (!isCommandResults(results) || !isBudgetState(budget)) {
      throw new Error('Invalid command results from wasm');
    }
    if (results.length === 0) {
      return;
    }
    const ids = changedIds(results);
    const delta = ids === null ? null : collectDelta(this.engine, ids);
    if (delta !== null) {
      this.lastSignals = undefined;
    }
    const message: CommandResultsMessage = { type: 'commandResults', results, delta, budget };
    this.post(message, delta === null ? [] : deltaTransfer(delta));
  }

  private postSnapshot(): void {
    const count = this.engine.fillSnapshot();
    this.views.refresh();
    const source = this.views.views;
    const target = this.pool.take(count);
    target.ids.set(source.ids.subarray(0, count));
    target.x.set(source.x.subarray(0, count));
    target.y.set(source.y.subarray(0, count));
    target.heading.set(source.heading.subarray(0, count));
    target.style.set(source.style.subarray(0, count));
    const tick = this.engine.tick();
    const message: SnapshotMessage = {
      type: 'snapshot',
      tick,
      simTime: tick * DT,
      count,
      buffers: target,
    };
    this.post(message, snapshotTransfer(target));
  }

  private postStats(): void {
    const stats: unknown = this.engine.stats();
    if (!isStatsSnapshot(stats)) {
      throw new Error('Invalid stats from wasm');
    }
    const roadSpeedRatio = new Float32Array(this.engine.roadSpeedRatio());
    const message: StatsMessage = { type: 'stats', stats, roadSpeedRatio };
    this.post(message, [roadSpeedRatio.buffer]);
  }
}
