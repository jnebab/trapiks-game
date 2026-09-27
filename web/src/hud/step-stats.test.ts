import { describe, expect, it } from 'vitest';
import { StepStats, perfFrom } from './step-stats';

describe('StepStats', () => {
  it('ignores samples before the start tick', () => {
    const steps = new StepStats(100);
    steps.record(50, Float64Array.from([99]));
    steps.record(100, Float64Array.from([1, 2, 3, 4]));
    expect(steps.summary()).toEqual({
      stepCount: '4',
      stepP50: '3.00',
      stepP95: '4.00',
      stepMax: '4.00',
    });
  });

  it('reads the start tick from the query', () => {
    expect(perfFrom('?perfFrom=9000')).toBe(9000);
    expect(perfFrom('?perf=1')).toBe(0);
  });
});
