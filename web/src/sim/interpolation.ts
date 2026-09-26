import type { SnapshotBuffers, SnapshotMessage } from './protocol';

const BLOCK = 4096;
const MIN_INTERVAL_MS = 1;
const MAX_INTERVAL_MS = 250;
const TAU = Math.PI * 2;

export interface VehicleFrame {
  count: number;
  x: Float32Array;
  y: Float32Array;
  heading: Float32Array;
  style: Uint8Array;
}

interface Received {
  message: SnapshotMessage;
  receivedAt: number;
}

export function createFrame(): VehicleFrame {
  return {
    count: 0,
    x: new Float32Array(0),
    y: new Float32Array(0),
    heading: new Float32Array(0),
    style: new Uint8Array(0),
  };
}

function ensureCapacity(frame: VehicleFrame, count: number): void {
  frame.count = count;
  if (frame.x.length >= count) {
    return;
  }
  const capacity = Math.ceil(count / BLOCK) * BLOCK;
  frame.x = new Float32Array(capacity);
  frame.y = new Float32Array(capacity);
  frame.heading = new Float32Array(capacity);
  frame.style = new Uint8Array(capacity);
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

export function shortestArc(from: number, to: number): number {
  return ((((to - from + Math.PI) % TAU) + TAU) % TAU) - Math.PI;
}

function copyCurrent(out: VehicleFrame, current: SnapshotBuffers, count: number): void {
  out.x.set(current.x.subarray(0, count));
  out.y.set(current.y.subarray(0, count));
  out.heading.set(current.heading.subarray(0, count));
  out.style.set(current.style.subarray(0, count));
}

function blendVehicle(
  out: VehicleFrame,
  pair: { a: SnapshotBuffers; b: SnapshotBuffers; alpha: number },
  i: number,
  j: number,
): void {
  const { a, b, alpha } = pair;
  const ax = a.x[j] ?? 0;
  const ay = a.y[j] ?? 0;
  const ah = a.heading[j] ?? 0;
  out.x[i] = ax + alpha * ((b.x[i] ?? 0) - ax);
  out.y[i] = ay + alpha * ((b.y[i] ?? 0) - ay);
  out.heading[i] = ah + alpha * shortestArc(ah, b.heading[i] ?? 0);
}

function indexById(snapshot: SnapshotMessage): Map<number, number> {
  const byId = new Map<number, number>();
  for (let j = 0; j < snapshot.count; j += 1) {
    byId.set(snapshot.buffers.ids[j] ?? 0, j);
  }
  return byId;
}

function fillPrevIndex(
  prevIndex: Int32Array,
  byId: Map<number, number>,
  current: SnapshotMessage,
): void {
  const ids = current.buffers.ids;
  for (let i = 0; i < current.count; i += 1) {
    prevIndex[i] = byId.get(ids[i] ?? 0) ?? -1;
  }
}

export class SnapshotHistory {
  private previous: Received | undefined;
  private current: Received | undefined;
  private prevIndex = new Int32Array(0);

  push(message: SnapshotMessage, now: number): SnapshotMessage | undefined {
    const evicted = this.previous;
    this.previous = this.current;
    this.current = { message, receivedAt: now };
    this.buildIndex();
    return evicted?.message;
  }

  sample(now: number, out: VehicleFrame): void {
    const current = this.current;
    if (!current) {
      out.count = 0;
      return;
    }
    const count = current.message.count;
    ensureCapacity(out, count);
    copyCurrent(out, current.message.buffers, count);
    if (this.previous) {
      this.blend(now, out, this.previous, current);
    }
  }

  private blend(now: number, out: VehicleFrame, previous: Received, current: Received): void {
    const interval = clamp(
      current.receivedAt - previous.receivedAt,
      MIN_INTERVAL_MS,
      MAX_INTERVAL_MS,
    );
    const alpha = clamp((now - current.receivedAt) / interval, 0, 1);
    const pair = { a: previous.message.buffers, b: current.message.buffers, alpha };
    for (let i = 0; i < current.message.count; i += 1) {
      const j = this.prevIndex[i] ?? -1;
      if (j >= 0) {
        blendVehicle(out, pair, i, j);
      }
    }
  }

  private buildIndex(): void {
    const current = this.current?.message;
    const previous = this.previous?.message;
    if (!current || !previous) {
      return;
    }
    if (this.prevIndex.length < current.count) {
      this.prevIndex = new Int32Array(Math.ceil(current.count / BLOCK) * BLOCK);
    }
    fillPrevIndex(this.prevIndex, indexById(previous), current);
  }
}
