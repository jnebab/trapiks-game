import { describe, expect, it } from 'vitest';
import { pillTint, pillsVisible } from './signal-pill-style';
import { STREET_MIN_SCALE } from './style';

describe('pillTint', () => {
  it('maps green, amber and red codes', () => {
    expect(pillTint(0)).toBe(0x43b581);
    expect(pillTint(1)).toBe(0xf2b33d);
    expect(pillTint(2)).toBe(0xe5484d);
  });

  it('falls back to red for unknown codes', () => {
    expect(pillTint(9)).toBe(0xe5484d);
  });
});

describe('pillsVisible', () => {
  it('shows pills only at street scale', () => {
    expect(pillsVisible(STREET_MIN_SCALE)).toBe(true);
    expect(pillsVisible(STREET_MIN_SCALE * 3)).toBe(true);
    expect(pillsVisible(STREET_MIN_SCALE - 0.01)).toBe(false);
  });
});
