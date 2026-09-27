import type { RoadArrays } from '../sim/protocol';
import { grow, makeF32, makeI8, makeU32, makeU8 } from './growable';
import type { Point } from './polyline';

export interface RoadRows {
  ids: Uint32Array;
  classCode: Uint8Array;
  lanesForward: Uint8Array;
  lanesBackward: Uint8Array;
  layer: Int8Array;
  name: Uint32Array;
  roundabout: Uint8Array;
  deleted: Uint8Array;
  from: Uint32Array;
  to: Uint32Array;
  pointLen: Uint32Array;
  x: Float32Array;
  y: Float32Array;
}

function at(column: ArrayLike<number>, i: number): number {
  return column[i] ?? 0;
}

export class RoadStore {
  count = 0;
  xs = new Float32Array(0);
  ys = new Float32Array(0);
  private pointCount = 0;
  private classCodes = new Uint8Array(0);
  private forward = new Uint8Array(0);
  private backward = new Uint8Array(0);
  private layers = new Int8Array(0);
  private names = new Uint32Array(0);
  private roundabouts = new Uint8Array(0);
  private deleted = new Uint8Array(0);
  private fromNodes = new Uint32Array(0);
  private toNodes = new Uint32Array(0);
  private starts = new Uint32Array(0);
  private lens = new Uint32Array(0);
  private nodeLayers: Map<number, [number, number]> | undefined;
  private ringNodes: Set<number> | undefined;

  static fromArrays(roads: RoadArrays): RoadStore {
    const store = new RoadStore();
    const count = roads.layer.length;
    const pointLen = Uint32Array.from({ length: count }, (_, road) => {
      const start = roads.pointStart[road] ?? 0;
      return (roads.pointStart[road + 1] ?? start) - start;
    });
    store.applyRoads({
      ...roads,
      ids: Uint32Array.from({ length: count }, (_, road) => road),
      deleted: new Uint8Array(count),
      pointLen,
    });
    return store;
  }

  classCode(road: number): number {
    return this.classCodes[road] ?? 0;
  }

  lanesForward(road: number): number {
    return this.forward[road] ?? 0;
  }

  lanesBackward(road: number): number {
    return this.backward[road] ?? 0;
  }

  lanes(road: number): number {
    return this.lanesForward(road) + this.lanesBackward(road);
  }

  layer(road: number): number {
    return this.layers[road] ?? 0;
  }

  name(road: number): number {
    return this.names[road] ?? 0;
  }

  isRoundabout(road: number): boolean {
    return this.roundabouts[road] === 1;
  }

  isDeleted(road: number): boolean {
    return road >= this.count || this.deleted[road] === 1;
  }

  from(road: number): number {
    return this.fromNodes[road] ?? 0;
  }

  to(road: number): number {
    return this.toNodes[road] ?? 0;
  }

  pointRange(road: number): [number, number] {
    const start = this.starts[road] ?? 0;
    return [start, start + (this.lens[road] ?? 0)];
  }

  pointsOf(road: number): Point[] {
    const [start, end] = this.pointRange(road);
    const out: Point[] = [];
    for (let i = start; i < end; i += 1) {
      out.push({ x: this.xs[i] ?? 0, y: this.ys[i] ?? 0 });
    }
    return out;
  }

  isRingNode(node: number): boolean {
    this.ringNodes ??= this.collectRingNodes();
    return this.ringNodes.has(node);
  }

  isMixedNode(node: number): boolean {
    this.nodeLayers ??= this.collectNodeLayers();
    const [low, high] = this.nodeLayers.get(node) ?? [0, 0];
    return low !== high;
  }

  truncate(count: number): void {
    this.count = Math.min(this.count, count);
    this.forgetNodeTraits();
  }

  applyRoads(rows: RoadRows): void {
    this.forgetNodeTraits();
    const maxId = rows.ids.reduce((max, id) => Math.max(max, id), -1);
    this.reserve(maxId + 1);
    let offset = 0;
    rows.ids.forEach((road, i) => {
      this.setRow(road, rows, i);
      offset = this.setPoints(road, rows, offset, rows.pointLen[i] ?? 0);
    });
  }

  private setRow(road: number, rows: RoadRows, i: number): void {
    this.classCodes[road] = at(rows.classCode, i);
    this.forward[road] = at(rows.lanesForward, i);
    this.backward[road] = at(rows.lanesBackward, i);
    this.layers[road] = at(rows.layer, i);
    this.names[road] = at(rows.name, i);
    this.roundabouts[road] = at(rows.roundabout, i);
    this.deleted[road] = at(rows.deleted, i);
    this.fromNodes[road] = at(rows.from, i);
    this.toNodes[road] = at(rows.to, i);
  }

  private setPoints(road: number, rows: RoadRows, offset: number, len: number): number {
    const start = this.pointCount;
    this.xs = grow(this.xs, start + len, makeF32);
    this.ys = grow(this.ys, start + len, makeF32);
    this.xs.set(rows.x.subarray(offset, offset + len), start);
    this.ys.set(rows.y.subarray(offset, offset + len), start);
    this.pointCount += len;
    this.starts[road] = start;
    this.lens[road] = len;
    return offset + len;
  }

  private forgetNodeTraits(): void {
    this.ringNodes = undefined;
    this.nodeLayers = undefined;
  }

  private collectNodeLayers(): Map<number, [number, number]> {
    const layers = new Map<number, [number, number]>();
    for (let road = 0; road < this.count; road += 1) {
      if (this.isDeleted(road)) {
        continue;
      }
      const layer = this.layer(road);
      for (const node of [this.from(road), this.to(road)]) {
        const [low, high] = layers.get(node) ?? [layer, layer];
        layers.set(node, [Math.min(low, layer), Math.max(high, layer)]);
      }
    }
    return layers;
  }

  private collectRingNodes(): Set<number> {
    const nodes = new Set<number>();
    for (let road = 0; road < this.count; road += 1) {
      if (this.isRoundabout(road) && !this.isDeleted(road)) {
        nodes.add(this.from(road));
        nodes.add(this.to(road));
      }
    }
    return nodes;
  }

  private reserve(count: number): void {
    this.classCodes = grow(this.classCodes, count, makeU8);
    this.forward = grow(this.forward, count, makeU8);
    this.backward = grow(this.backward, count, makeU8);
    this.layers = grow(this.layers, count, makeI8);
    this.names = grow(this.names, count, makeU32);
    this.roundabouts = grow(this.roundabouts, count, makeU8);
    this.deleted = grow(this.deleted, count, makeU8);
    this.fromNodes = grow(this.fromNodes, count, makeU32);
    this.toNodes = grow(this.toNodes, count, makeU32);
    this.starts = grow(this.starts, count, makeU32);
    this.lens = grow(this.lens, count, makeU32);
    this.count = Math.max(this.count, count);
  }
}
