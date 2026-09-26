import type { NodeArrays } from '../sim/protocol';
import { grow, makeF32, makeU8 } from './growable';

export interface NodeRows {
  ids: Uint32Array;
  x: Float32Array;
  y: Float32Array;
  control: Uint8Array;
}

export class NodeStore {
  count = 0;
  private xs = new Float32Array(0);
  private ys = new Float32Array(0);
  private controls = new Uint8Array(0);

  static fromArrays(nodes: NodeArrays): NodeStore {
    const store = new NodeStore();
    const ids = Uint32Array.from({ length: nodes.x.length }, (_, node) => node);
    store.applyNodes({ ids, x: nodes.x, y: nodes.y, control: nodes.controlCode });
    return store;
  }

  x(node: number): number {
    return this.xs[node] ?? 0;
  }

  y(node: number): number {
    return this.ys[node] ?? 0;
  }

  control(node: number): number {
    return this.controls[node] ?? 0;
  }

  truncate(count: number): void {
    this.count = Math.min(this.count, count);
  }

  applyNodes(rows: NodeRows): void {
    const needed = rows.ids.reduce((max, id) => Math.max(max, id + 1), 0);
    this.xs = grow(this.xs, needed, makeF32);
    this.ys = grow(this.ys, needed, makeF32);
    this.controls = grow(this.controls, needed, makeU8);
    this.count = Math.max(this.count, needed);
    rows.ids.forEach((node, i) => {
      this.xs[node] = rows.x[i] ?? 0;
      this.ys[node] = rows.y[i] ?? 0;
      this.controls[node] = rows.control[i] ?? 0;
    });
  }
}
