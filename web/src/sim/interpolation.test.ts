import { describe, expect, it } from 'vitest';
import { SnapshotHistory, createFrame } from './interpolation';
import type { SnapshotMessage } from './protocol';

interface Car {
  id: number;
  x: number;
  heading?: number;
}

function snapshot(tick: number, cars: Car[]): SnapshotMessage {
  return {
    type: 'snapshot',
    session: 0,
    tick,
    simTime: tick / 10,
    count: cars.length,
    buffers: {
      ids: new Uint32Array(cars.map((c) => c.id)),
      x: new Float32Array(cars.map((c) => c.x)),
      y: new Float32Array(cars.map((c) => c.x * 2)),
      heading: new Float32Array(cars.map((c) => c.heading ?? 0)),
      style: new Uint8Array(cars.map((c) => c.id % 9)),
    },
  };
}

function sampled(history: SnapshotHistory, now: number): ReturnType<typeof createFrame> {
  const frame = createFrame();
  history.sample(now, frame);
  return frame;
}

describe('SnapshotHistory', () => {
  it('outputs the only snapshot unchanged', () => {
    const history = new SnapshotHistory();
    history.push(snapshot(1, [{ id: 3, x: 5 }]), 0);
    const frame = sampled(history, 50);
    expect(frame.count).toBe(1);
    expect(frame.x[0]).toBe(5);
    expect(frame.x.length).toBe(4096);
  });

  it('lerps position at alpha 0, 0.5 and 1', () => {
    const history = new SnapshotHistory();
    history.push(snapshot(1, [{ id: 1, x: 0 }]), 0);
    history.push(snapshot(2, [{ id: 1, x: 10 }]), 100);
    expect(sampled(history, 100).x[0]).toBeCloseTo(0);
    expect(sampled(history, 150).x[0]).toBeCloseTo(5);
    expect(sampled(history, 150).y[0]).toBeCloseTo(10);
    expect(sampled(history, 200).x[0]).toBeCloseTo(10);
    expect(sampled(history, 900).x[0]).toBeCloseTo(10);
  });

  it('wraps heading across ±π along the shortest arc', () => {
    const history = new SnapshotHistory();
    history.push(snapshot(1, [{ id: 1, x: 0, heading: 3.0 }]), 0);
    history.push(snapshot(2, [{ id: 1, x: 0, heading: -3.0 }]), 100);
    const heading = sampled(history, 150).heading[0] ?? 0;
    expect(heading).toBeCloseTo(Math.PI, 5);
  });
});

describe('SnapshotHistory identity', () => {
  it('snaps a new id to its current values', () => {
    const history = new SnapshotHistory();
    history.push(snapshot(1, [{ id: 1, x: 0 }]), 0);
    history.push(
      snapshot(2, [
        { id: 2, x: 40 },
        { id: 1, x: 10 },
      ]),
      100,
    );
    const frame = sampled(history, 150);
    expect(frame.count).toBe(2);
    expect(frame.x[0]).toBe(40);
    expect(frame.x[1]).toBeCloseTo(5);
  });

  it('returns the evicted snapshot', () => {
    const history = new SnapshotHistory();
    const first = snapshot(1, []);
    expect(history.push(first, 0)).toBeUndefined();
    expect(history.push(snapshot(2, []), 100)).toBeUndefined();
    expect(history.push(snapshot(3, []), 200)).toBe(first);
  });
});
