import type { ArrayCtor } from './values';

export interface RoadArrays {
  pointStart: Uint32Array;
  x: Float32Array;
  y: Float32Array;
  classCode: Uint8Array;
  lanesForward: Uint8Array;
  lanesBackward: Uint8Array;
  layer: Int8Array;
  name: Uint32Array;
  roundabout: Uint8Array;
  from: Uint32Array;
  to: Uint32Array;
}

export interface NodeArrays {
  x: Float32Array;
  y: Float32Array;
  controlCode: Uint8Array;
}

export interface AreaArrays {
  kindCode: Uint8Array;
  hole: Uint8Array;
  ringStart: Uint32Array;
  x: Float32Array;
  y: Float32Array;
}

export const roadShape: Record<keyof RoadArrays, ArrayCtor> = {
  pointStart: Uint32Array,
  x: Float32Array,
  y: Float32Array,
  classCode: Uint8Array,
  lanesForward: Uint8Array,
  lanesBackward: Uint8Array,
  layer: Int8Array,
  name: Uint32Array,
  roundabout: Uint8Array,
  from: Uint32Array,
  to: Uint32Array,
};

export const nodeShape: Record<keyof NodeArrays, ArrayCtor> = {
  x: Float32Array,
  y: Float32Array,
  controlCode: Uint8Array,
};

export const areaShape: Record<keyof AreaArrays, ArrayCtor> = {
  kindCode: Uint8Array,
  hole: Uint8Array,
  ringStart: Uint32Array,
  x: Float32Array,
  y: Float32Array,
};
