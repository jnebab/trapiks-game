import type { ApproachMarkerArrays, JunctionShapeArrays, NodeArrays } from '../../sim/protocol';
import type { Rect } from '../rect';

export interface StreetData {
  setbacks: Float32Array;
  junctions: JunctionShapeArrays;
  markers: ApproachMarkerArrays;
  nodes: NodeArrays;
}

export interface Owned {
  x: number;
  y: number;
  box: Rect;
}

function ringBox(junctions: JunctionShapeArrays, shape: number): Rect {
  const start = junctions.ringStart[shape] ?? 0;
  const end = junctions.ringStart[shape + 1] ?? start;
  const xs = Array.from(junctions.x.subarray(start, end));
  const ys = Array.from(junctions.y.subarray(start, end));
  return {
    minX: Math.min(...xs),
    minY: Math.min(...ys),
    maxX: Math.max(...xs),
    maxY: Math.max(...ys),
  };
}

export function junctionOwner(street: StreetData, shape: number): Owned {
  const node = street.junctions.node[shape] ?? 0;
  return {
    x: street.nodes.x[node] ?? 0,
    y: street.nodes.y[node] ?? 0,
    box: ringBox(street.junctions, shape),
  };
}

export function markerOwner(street: StreetData, marker: number): Owned {
  const m = street.markers;
  const x1 = m.x1[marker] ?? 0;
  const y1 = m.y1[marker] ?? 0;
  const x2 = m.x2[marker] ?? 0;
  const y2 = m.y2[marker] ?? 0;
  return {
    x: (x1 + x2) / 2,
    y: (y1 + y2) / 2,
    box: {
      minX: Math.min(x1, x2),
      minY: Math.min(y1, y2),
      maxX: Math.max(x1, x2),
      maxY: Math.max(y1, y2),
    },
  };
}
