import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { startWallTicks } from './loop';

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(0);
});

afterEach(() => {
  vi.useRealTimers();
});

describe('startWallTicks', () => {
  it('ticks once per period', () => {
    const onTick = vi.fn();
    const loop = startWallTicks(onTick, 100);
    vi.advanceTimersByTime(1000);
    expect(onTick).toHaveBeenCalledTimes(10);
    loop.stop();
    vi.advanceTimersByTime(1000);
    expect(onTick).toHaveBeenCalledTimes(10);
  });

  it('resyncs after a stall instead of bursting', () => {
    const times: number[] = [];
    let stallNext = false;
    const onTick = (): void => {
      times.push(Date.now());
      if (stallNext) {
        stallNext = false;
        vi.setSystemTime(Date.now() + 500);
      }
    };
    const loop = startWallTicks(onTick, 100);
    vi.advanceTimersByTime(100);
    stallNext = true;
    vi.advanceTimersByTime(100);
    vi.advanceTimersByTime(300);
    loop.stop();
    expect(times.slice(0, 2)).toEqual([100, 200]);
    const afterStall = times.slice(2);
    expect(afterStall[0]).toBeGreaterThanOrEqual(700);
    afterStall.slice(1).forEach((time, i) => {
      expect(time - (afterStall[i] ?? 0)).toBeGreaterThanOrEqual(99);
    });
  });
});
