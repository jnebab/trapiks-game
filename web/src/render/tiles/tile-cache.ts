import type { TileGraphics, TilePiece } from './tile-builder';

export const TILE_CACHE_CAPACITY = 256;

function setVisible(tile: TileGraphics, visible: boolean): void {
  for (const piece of tile.pieces) {
    piece.graphics.visible = visible;
  }
}

function destroyTile(tile: TileGraphics): void {
  for (const piece of tile.pieces) {
    piece.graphics.destroy();
  }
}

export class TileCache {
  private readonly tiles = new Map<string, TileGraphics>();
  private shown = new Set<string>();

  constructor(private readonly capacity = TILE_CACHE_CAPACITY) {}

  get size(): number {
    return this.tiles.size;
  }

  has(key: string): boolean {
    return this.tiles.has(key);
  }

  pending(keys: readonly string[]): string[] {
    return keys.filter((key) => !this.tiles.has(key));
  }

  get(key: string): TileGraphics | undefined {
    return this.tiles.get(key);
  }

  addMarkings(key: string, pieces: TilePiece[]): void {
    const tile = this.tiles.get(key);
    if (tile === undefined) {
      return;
    }
    tile.pieces.push(...pieces);
    tile.markings = true;
    setVisible(tile, this.shown.has(key));
  }

  insert(key: string, tile: TileGraphics): void {
    setVisible(tile, false);
    this.tiles.set(key, tile);
    this.evict(key);
  }

  show(keys: readonly string[]): void {
    this.shown = new Set(keys);
    for (const key of keys) {
      this.touch(key);
    }
    for (const [key, tile] of this.tiles) {
      setVisible(tile, this.shown.has(key));
    }
  }

  private touch(key: string): void {
    const tile = this.tiles.get(key);
    if (tile === undefined) {
      return;
    }
    this.tiles.delete(key);
    this.tiles.set(key, tile);
  }

  private evict(inserted: string): void {
    for (const [key, tile] of this.tiles) {
      if (this.tiles.size <= this.capacity) {
        return;
      }
      if (key !== inserted && !this.shown.has(key)) {
        this.tiles.delete(key);
        destroyTile(tile);
      }
    }
  }
}
