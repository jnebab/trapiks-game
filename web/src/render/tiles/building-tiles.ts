import type { Container } from 'pixi.js';
import { buildTileBuildings, type BuildingSources } from '../buildings/building-layer';
import { expand, intersects } from '../rect';
import { roadBox } from './road-bounds';
import type { TileCache } from './tile-cache';
import type { TileIndex } from './tile-index';

export const BUILDING_REACH = 40;

export interface BuildingBand {
  index: TileIndex;
  cache: TileCache;
}

export class BuildingTiles {
  constructor(
    private readonly sources: BuildingSources,
    private readonly container: Container,
  ) {}

  setVisible(visible: boolean): void {
    this.container.visible = visible;
  }

  ensure(band: BuildingBand, keys: readonly string[], budget: number): void {
    const missing = keys.filter((key) => {
      const tile = band.cache.get(key);
      return tile !== undefined && tile.buildings === undefined;
    });
    for (const key of missing.slice(0, budget)) {
      this.buildOne(band, key);
    }
  }

  near(index: TileIndex, roads: readonly number[]): Set<string> {
    const boxes = roads.map((road) => expand(roadBox(this.sources.roads, road), BUILDING_REACH));
    const keys = new Set<string>();
    for (const [key, entry] of index.entries) {
      if (boxes.some((box) => intersects(box, entry.bounds))) {
        keys.add(key);
      }
    }
    return keys;
  }

  private buildOne(band: BuildingBand, key: string): void {
    const entry = band.index.get(key);
    if (entry === undefined) {
      return;
    }
    const graphics = buildTileBuildings(this.sources, entry, this.container);
    band.cache.addBuildings(key, graphics);
  }
}
