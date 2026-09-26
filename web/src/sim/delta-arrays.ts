import { hasArrays, isRecord, type ArrayCtor } from './values';

export interface DeltaColumns {
  roadIds: Uint32Array;
  roadClass: Uint8Array;
  roadLanesForward: Uint8Array;
  roadLanesBackward: Uint8Array;
  roadLayer: Int8Array;
  roadName: Uint32Array;
  roadRoundabout: Uint8Array;
  roadDeleted: Uint8Array;
  roadFrom: Uint32Array;
  roadTo: Uint32Array;
  roadPointLen: Uint32Array;
  roadX: Float32Array;
  roadY: Float32Array;
  nodeIds: Uint32Array;
  nodeX: Float32Array;
  nodeY: Float32Array;
  nodeControl: Uint8Array;
  setbackIds: Uint32Array;
  setbackStart: Float32Array;
  setbackEnd: Float32Array;
  junctionNode: Uint32Array;
  junctionLayer: Int8Array;
  junctionMinLayer: Int8Array;
  junctionRingStart: Uint32Array;
  junctionX: Float32Array;
  junctionY: Float32Array;
  markerNodes: Uint32Array;
  markerLink: Uint32Array;
  markerNode: Uint32Array;
  markerKind: Uint8Array;
  markerX1: Float32Array;
  markerY1: Float32Array;
  markerX2: Float32Array;
  markerY2: Float32Array;
  pillLink: Uint32Array;
  pillX: Float32Array;
  pillY: Float32Array;
  pillAngle: Float32Array;
}

export interface DeltaArrays extends DeltaColumns {
  roadCount: number;
  nodeCount: number;
}

export const deltaShape: Record<keyof DeltaColumns, ArrayCtor> = {
  roadIds: Uint32Array,
  roadClass: Uint8Array,
  roadLanesForward: Uint8Array,
  roadLanesBackward: Uint8Array,
  roadLayer: Int8Array,
  roadName: Uint32Array,
  roadRoundabout: Uint8Array,
  roadDeleted: Uint8Array,
  roadFrom: Uint32Array,
  roadTo: Uint32Array,
  roadPointLen: Uint32Array,
  roadX: Float32Array,
  roadY: Float32Array,
  nodeIds: Uint32Array,
  nodeX: Float32Array,
  nodeY: Float32Array,
  nodeControl: Uint8Array,
  setbackIds: Uint32Array,
  setbackStart: Float32Array,
  setbackEnd: Float32Array,
  junctionNode: Uint32Array,
  junctionLayer: Int8Array,
  junctionMinLayer: Int8Array,
  junctionRingStart: Uint32Array,
  junctionX: Float32Array,
  junctionY: Float32Array,
  markerNodes: Uint32Array,
  markerLink: Uint32Array,
  markerNode: Uint32Array,
  markerKind: Uint8Array,
  markerX1: Float32Array,
  markerY1: Float32Array,
  markerX2: Float32Array,
  markerY2: Float32Array,
  pillLink: Uint32Array,
  pillX: Float32Array,
  pillY: Float32Array,
  pillAngle: Float32Array,
};

export const deltaColumnNames = Object.keys(deltaShape) as (keyof DeltaColumns)[];

export function isDeltaArrays(value: unknown): value is DeltaArrays {
  if (!isRecord(value)) {
    return false;
  }
  const counts = typeof value.roadCount === 'number' && typeof value.nodeCount === 'number';
  return counts && hasArrays(value, deltaShape);
}

export function deltaTransfer(delta: DeltaArrays): ArrayBuffer[] {
  return deltaColumnNames.map((name) => delta[name].buffer as ArrayBuffer);
}
