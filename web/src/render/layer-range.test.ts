import { describe, expect, it } from 'vitest';
import { clampLayer, layerIndex, MAX_LAYER, MIN_LAYER } from './layer-range';

describe('layerIndex', () => {
  it('maps each layer to its own zero-based slot', () => {
    expect(layerIndex(MIN_LAYER)).toBe(0);
    expect(layerIndex(0)).toBe(-MIN_LAYER);
    expect(layerIndex(1)).toBe(layerIndex(0) + 1);
  });

  it('clamps layers outside the drawn range', () => {
    expect(layerIndex(MAX_LAYER + 4)).toBe(MAX_LAYER - MIN_LAYER);
    expect(layerIndex(MIN_LAYER - 4)).toBe(0);
    expect(clampLayer(MAX_LAYER + 1)).toBe(MAX_LAYER);
  });
});
