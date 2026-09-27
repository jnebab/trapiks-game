import { describe, expect, it } from 'vitest';
import { clampAboveDock } from './road-tooltip';

describe('clampAboveDock', () => {
  it('keeps a tooltip that already clears the dock', () => {
    expect(clampAboveDock(100, 40, 600)).toBe(100);
  });

  it('lifts a tooltip that would cover the dock', () => {
    expect(clampAboveDock(580, 40, 600)).toBe(552);
  });
});
