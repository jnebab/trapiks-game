import type { SnapshotBuffers } from '../sim/protocol';

export interface SnapshotLayout {
  ids: number;
  x: number;
  y: number;
  heading: number;
  style: number;
  layer: number;
  capacity: number;
}

function buildViews(buffer: ArrayBuffer, layout: SnapshotLayout): SnapshotBuffers {
  const length = layout.capacity;
  return {
    ids: new Uint32Array(buffer, layout.ids, length),
    x: new Float32Array(buffer, layout.x, length),
    y: new Float32Array(buffer, layout.y, length),
    heading: new Float32Array(buffer, layout.heading, length),
    style: new Uint8Array(buffer, layout.style, length),
    layer: new Int8Array(buffer, layout.layer, length),
  };
}

export class MemoryViews {
  private buffer: ArrayBuffer;
  private current: SnapshotBuffers;

  constructor(
    private readonly memory: WebAssembly.Memory,
    private readonly layout: SnapshotLayout,
  ) {
    this.buffer = memory.buffer;
    this.current = buildViews(this.buffer, layout);
  }

  refresh(): void {
    if (this.memory.buffer === this.buffer) {
      return;
    }
    this.buffer = this.memory.buffer;
    this.current = buildViews(this.buffer, this.layout);
  }

  get views(): SnapshotBuffers {
    return this.current;
  }
}
