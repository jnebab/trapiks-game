import { Graphics, type Container } from 'pixi.js';
import type { DetailStore } from './detail-store';

const HIGHLIGHT = 0x8ed8f6;
const RING_WIDTH = 0.6;
const RING_ALPHA = 0.8;

export class NodeHighlight {
  node: number | undefined;
  private readonly graphics = new Graphics();

  constructor(
    container: Container,
    private readonly detail: DetailStore,
  ) {
    container.addChild(this.graphics);
  }

  set(node: number | undefined): void {
    this.node = node;
    this.draw();
  }

  draw(): void {
    this.graphics.clear();
    const ring = this.node === undefined ? undefined : this.detail.junctions.get(this.node);
    if (ring === undefined) {
      return;
    }
    this.graphics
      .poly(ring.points, true)
      .stroke({ width: RING_WIDTH, color: HIGHLIGHT, alpha: RING_ALPHA, join: 'round' });
  }
}
