export const MIN_LAYER = -3;
export const MAX_LAYER = 5;

export function clampLayer(layer: number): number {
  return Math.min(Math.max(layer, MIN_LAYER), MAX_LAYER);
}

export function layerIndex(layer: number): number {
  return clampLayer(layer) - MIN_LAYER;
}
