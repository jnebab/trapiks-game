import type { ArrayCtor } from './values';

export interface JunctionShapeArrays {
  node: Uint32Array;
  layer: Int8Array;
  minLayer: Int8Array;
  ringStart: Uint32Array;
  x: Float32Array;
  y: Float32Array;
}

export interface ApproachMarkerArrays {
  link: Uint32Array;
  node: Uint32Array;
  kind: Uint8Array;
  x1: Float32Array;
  y1: Float32Array;
  x2: Float32Array;
  y2: Float32Array;
}

export const junctionShape: Record<keyof JunctionShapeArrays, ArrayCtor> = {
  node: Uint32Array,
  layer: Int8Array,
  minLayer: Int8Array,
  ringStart: Uint32Array,
  x: Float32Array,
  y: Float32Array,
};

export const markerShape: Record<keyof ApproachMarkerArrays, ArrayCtor> = {
  link: Uint32Array,
  node: Uint32Array,
  kind: Uint8Array,
  x1: Float32Array,
  y1: Float32Array,
  x2: Float32Array,
  y2: Float32Array,
};
