import { describe, expect, it } from 'vitest';
import { isMainMessage } from './protocol';

function buffers(): Record<string, unknown> {
  return {
    ids: new Uint32Array(4),
    x: new Float32Array(4),
    y: new Float32Array(4),
    heading: new Float32Array(4),
    style: new Uint8Array(4),
  };
}

describe('isMainMessage', () => {
  const config = { seed: 1, mode: 'City', vehicles_per_hour: 3000 };

  it('accepts load with a city or region config', () => {
    expect(isMainMessage({ type: 'load', url: 'a', config })).toBe(true);
    const region = { ...config, mode: { Region: { center_x: 0, center_y: 0, radius: 5 } } };
    expect(isMainMessage({ type: 'load', url: 'a', config: region })).toBe(true);
  });

  it('rejects load with a missing or malformed config', () => {
    expect(isMainMessage({ type: 'load', url: 'a' })).toBe(false);
    expect(isMainMessage({ type: 'load', url: 'a', config: { ...config, mode: 'Town' } })).toBe(
      false,
    );
    expect(isMainMessage({ type: 'load', url: 'a', config: { ...config, seed: 1n } })).toBe(false);
  });

  it('accepts only known speeds', () => {
    for (const speed of [0, 1, 2, 4, 8, 'max']) {
      expect(isMainMessage({ type: 'speed', speed })).toBe(true);
    }
    expect(isMainMessage({ type: 'speed', speed: 3 })).toBe(false);
    expect(isMainMessage({ type: 'speed', speed: 'fast' })).toBe(false);
  });

  it('accepts returned buffers and rejects unknown types', () => {
    expect(isMainMessage({ type: 'buffers', buffers: buffers() })).toBe(true);
    expect(isMainMessage({ type: 'buffers', buffers: {} })).toBe(false);
    expect(isMainMessage({ type: 'toString' })).toBe(false);
    expect(isMainMessage({ type: 'snapshot', session: 0 })).toBe(false);
  });
});
