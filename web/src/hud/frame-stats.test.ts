import { describe, expect, it } from 'vitest';
import { FRAME_WINDOW, FrameStats, perfEnabled } from './frame-stats';

describe('FrameStats', () => {
  it('reports percentiles over the rolling window', () => {
    const stats = new FrameStats();
    for (let i = 1; i <= 100; i += 1) {
      stats.record(i);
    }
    expect(stats.percentile(0.5)).toBe(51);
    expect(stats.percentile(0.95)).toBe(96);
  });

  it('forgets samples older than the window', () => {
    const stats = new FrameStats();
    for (let i = 0; i < FRAME_WINDOW; i += 1) {
      stats.record(100);
    }
    for (let i = 0; i < FRAME_WINDOW; i += 1) {
      stats.record(1);
    }
    expect(stats.percentile(0.95)).toBe(1);
  });

  it('reads the perf flag from the query', () => {
    expect(perfEnabled('?perf=1')).toBe(true);
    expect(perfEnabled('?map=synthetic')).toBe(false);
  });
});
