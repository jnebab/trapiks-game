import { Graphics } from 'pixi.js';
import type { RoadPass } from '../layers';

export interface TilePiece {
  layer: number;
  pass: RoadPass;
  graphics: Graphics;
}

export class PieceSet {
  private readonly pieces = new Map<string, TilePiece>();

  get(layer: number, pass: RoadPass): Graphics {
    const key = `${String(layer)}|${pass}`;
    const existing = this.pieces.get(key);
    if (existing !== undefined) {
      return existing.graphics;
    }
    const graphics = new Graphics();
    this.pieces.set(key, { layer, pass, graphics });
    return graphics;
  }

  list(): TilePiece[] {
    return [...this.pieces.values()];
  }
}
