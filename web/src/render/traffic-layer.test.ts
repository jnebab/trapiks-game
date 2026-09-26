import { describe, expect, it } from 'vitest';
import { trafficClass } from './traffic-layer';

describe('trafficClass', () => {
  it('treats ratios at or above 0.8 as free flow', () => {
    expect(trafficClass(1)).toBe('free');
    expect(trafficClass(0.8)).toBe('free');
  });

  it('marks ratios from 0.5 up to 0.8 as slow', () => {
    expect(trafficClass(0.79)).toBe('slow');
    expect(trafficClass(0.5)).toBe('slow');
  });

  it('marks ratios below 0.5 as jammed', () => {
    expect(trafficClass(0.49)).toBe('jammed');
    expect(trafficClass(0)).toBe('jammed');
  });
});
