import type { Engine } from '../wasm/pkg/trapiks_sim_wasm.js';
import type { SnapshotMessage, Speed, StatsMessage, WorkerMessage } from '../sim/protocol';
import { isStatsSnapshot, snapshotTransfer, type SnapshotBuffers } from '../sim/values';
import { BufferPool } from './buffer-pool';
import { MemoryViews } from './memory-views';

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

  constructor(
    private readonly engine: Engine,
    memory: WebAssembly.Memory,
    private readonly post: Post,
  ) {
    this.views = new MemoryViews(memory, snapshotLayout(engine));
  }

  readonly onTick = (): void => {
    this.runSteps();
    this.postSnapshot();
    this.wallTicks += 1;
    if (this.wallTicks % STATS_EVERY === 0) {
      this.postStats();
    }
  };

  returnBuffers(buffers: SnapshotBuffers): void {
    this.pool.give(buffers);
  }

  private runSteps(): void {
    if (this.speed === 0) {
      return;
    }
    const limit = this.speed === 'max' ? Infinity : this.speed;
    const start = performance.now();
    for (let step = 0; step < limit; step += 1) {
      this.engine.step();
      if (performance.now() - start >= STEP_BUDGET_MS) {
        return;
      }
    }
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
