import type { ApproachMarkerArrays, JunctionShapeArrays } from '../sim/protocol';
import { grow, makeF32 } from './growable';

export interface JunctionRing {
  node: number;
  layer: number;
  minLayer: number;
  points: number[];
}

export interface Marker {
  link: number;
  node: number;
  kind: number;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

export interface SetbackRows {
  ids: Uint32Array;
  start: Float32Array;
  end: Float32Array;
}

export interface MarkerChange {
  removed: number[];
  added: number[];
}

function ringPoints(shapes: JunctionShapeArrays, shape: number): number[] {
  const start = shapes.ringStart[shape] ?? 0;
  const end = shapes.ringStart[shape + 1] ?? start;
  const out: number[] = [];
  for (let i = start; i < end; i += 1) {
    out.push(shapes.x[i] ?? 0, shapes.y[i] ?? 0);
  }
  return out;
}

function markerAt(markers: ApproachMarkerArrays, i: number): Marker {
  return {
    link: markers.link[i] ?? 0,
    node: markers.node[i] ?? 0,
    kind: markers.kind[i] ?? 0,
    x1: markers.x1[i] ?? 0,
    y1: markers.y1[i] ?? 0,
    x2: markers.x2[i] ?? 0,
    y2: markers.y2[i] ?? 0,
  };
}

export class DetailStore {
  readonly junctions = new Map<number, JunctionRing>();
  readonly markers = new Map<number, Marker>();
  private setbacks = new Float32Array(0);
  private readonly markersByNode = new Map<number, number[]>();
  private nextMarker = 0;

  static fromArrays(
    setbacks: Float32Array,
    shapes: JunctionShapeArrays,
    markers: ApproachMarkerArrays,
  ): DetailStore {
    const store = new DetailStore();
    store.setbacks = setbacks.slice();
    store.applyJunctions(shapes);
    for (let i = 0; i < markers.link.length; i += 1) {
      store.addMarker(markerAt(markers, i));
    }
    return store;
  }

  setback(road: number, end: 0 | 1): number {
    return this.setbacks[road * 2 + end] ?? 0;
  }

  applySetbacks(rows: SetbackRows): void {
    const needed = rows.ids.reduce((max, id) => Math.max(max, id * 2 + 2), 0);
    this.setbacks = grow(this.setbacks, needed, makeF32);
    rows.ids.forEach((road, i) => {
      this.setbacks[road * 2] = rows.start[i] ?? 0;
      this.setbacks[road * 2 + 1] = rows.end[i] ?? 0;
    });
  }

  applyJunctions(shapes: JunctionShapeArrays): void {
    shapes.node.forEach((node, shape) => {
      const points = ringPoints(shapes, shape);
      if (points.length === 0) {
        this.junctions.delete(node);
        return;
      }
      const layer = shapes.layer[shape] ?? 0;
      this.junctions.set(node, { node, layer, minLayer: shapes.minLayer[shape] ?? 0, points });
    });
  }

  markersAt(node: number): readonly number[] {
    return this.markersByNode.get(node) ?? [];
  }

  replaceMarkers(nodes: Uint32Array, markers: ApproachMarkerArrays): MarkerChange {
    const removed = Array.from(nodes).flatMap((node) => this.removeMarkersAt(node));
    const added: number[] = [];
    for (let i = 0; i < markers.link.length; i += 1) {
      added.push(this.addMarker(markerAt(markers, i)));
    }
    return { removed, added };
  }

  truncateNodes(count: number): MarkerChange {
    const nodes = [...this.markersByNode.keys(), ...this.junctions.keys()].filter(
      (n) => n >= count,
    );
    for (const node of nodes) {
      this.junctions.delete(node);
    }
    return { removed: nodes.flatMap((node) => this.removeMarkersAt(node)), added: [] };
  }

  private removeMarkersAt(node: number): number[] {
    const ids = this.markersByNode.get(node) ?? [];
    this.markersByNode.delete(node);
    for (const id of ids) {
      this.markers.delete(id);
    }
    return ids;
  }

  private addMarker(marker: Marker): number {
    const id = this.nextMarker;
    this.nextMarker += 1;
    this.markers.set(id, marker);
    const list = this.markersByNode.get(marker.node) ?? [];
    list.push(id);
    this.markersByNode.set(marker.node, list);
    return id;
  }
}
