import { describe, expect, it } from 'vitest';
import { statesChanged } from './signal-diff';

describe('statesChanged', () => {
  it('treats the first array as a change', () => {
    expect(statesChanged(undefined, new Uint8Array(0))).toBe(true);
  });

  it('detects equal and differing states', () => {
    const a = new Uint8Array([0, 2, 1]);
    expect(statesChanged(a, new Uint8Array([0, 2, 1]))).toBe(false);
    expect(statesChanged(a, new Uint8Array([0, 2, 2]))).toBe(true);
    expect(statesChanged(a, new Uint8Array([0, 2]))).toBe(true);
  });
});
