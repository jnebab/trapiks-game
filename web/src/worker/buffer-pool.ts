import type { SnapshotBuffers } from '../sim/protocol';

const MAX_SETS = 3;
const BLOCK = 4096;

function allocate(capacity: number): SnapshotBuffers {
  return {
    ids: new Uint32Array(capacity),
    x: new Float32Array(capacity),
    y: new Float32Array(capacity),
    heading: new Float32Array(capacity),
    style: new Uint8Array(capacity),
  };
}

export function roundedCapacity(count: number): number {
  return Math.max(1, Math.ceil(count / BLOCK)) * BLOCK;
}

export class BufferPool {
  private readonly sets: SnapshotBuffers[] = [];
  private lastTake = 0;

  take(count: number): SnapshotBuffers {
    this.lastTake = count;
    const index = this.sets.findIndex((set) => set.ids.length >= count);
    const reused = index >= 0 ? this.sets.splice(index, 1)[0] : undefined;
    return reused ?? allocate(roundedCapacity(count));
  }

  give(buffers: SnapshotBuffers): void {
    if (buffers.ids.length < this.lastTake || this.sets.length >= MAX_SETS) {
      return;
    }
    this.sets.push(buffers);
  }

  get size(): number {
    return this.sets.length;
  }
}
