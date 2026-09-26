import { Container, Graphics } from 'pixi.js';
import type { RoadStore } from './road-store';
import type { ActiveTiles, TileManager } from './tiles/tile-manager';
import type { BandName, TileEntry } from './tiles/tile-index';
import { tracePolyline } from './tiles/trace';

export type TrafficClass = 'free' | 'slow' | 'jammed';

const FREE_MIN = 0.8;
const SLOW_MIN = 0.5;
const WIDTH_FACTOR = 0.6;
const ALPHA = 0.85;
const BUILDS_PER_FRAME = 4;
const MAX_TILES = 256;
const COLORS: Readonly<Record<Exclude<TrafficClass, 'free'>, number>> = {
  slow: 0xf2b33d,
  jammed: 0xe5484d,
};

export function trafficClass(ratio: number): TrafficClass {
  if (ratio >= FREE_MIN) {
    return 'free';
  }
  return ratio >= SLOW_MIN ? 'slow' : 'jammed';
}

interface Stroke {
  color: number;
  width: number;
  roads: number[];
}

function groupStrokes(entry: TileEntry, view: ActiveTiles, ratios: Float32Array, roads: RoadStore) {
  const groups = new Map<string, Stroke>();
  for (const road of entry.roads) {
    const kind = trafficClass(ratios[road] ?? 1);
    if (kind === 'free' || roads.isDeleted(road)) {
      continue;
    }
    const color = COLORS[kind];
    const width = view.width(road) * WIDTH_FACTOR;
    const key = `${String(color)}|${String(width)}`;
    const group = groups.get(key) ?? { color, width, roads: [] };
    groups.set(key, group);
    group.roads.push(road);
  }
  return [...groups.values()];
}

function drawTile(entry: TileEntry, view: ActiveTiles, ratios: Float32Array, roads: RoadStore) {
  const g = new Graphics();
  for (const stroke of groupStrokes(entry, view, ratios, roads)) {
    for (const road of stroke.roads) {
      tracePolyline(g, roads, road);
    }
    g.stroke({
      width: stroke.width,
      color: stroke.color,
      alpha: ALPHA,
      join: 'round',
      cap: 'round',
    });
  }
  return g;
}

export class TrafficLayer {
  enabled = false;
  private ratios: Float32Array | undefined;
  private band: BandName | undefined;
  private readonly tiles = new Map<string, Graphics>();
  private dirty = new Set<string>();

  constructor(
    readonly container: Container,
    private readonly roads: RoadStore,
    private readonly source: TileManager,
  ) {}

  get visible(): boolean {
    return this.container.visible;
  }

  setRatios(ratios: Float32Array): void {
    this.ratios = ratios;
    this.dirty = new Set(this.tiles.keys());
  }

  update(): void {
    const view = this.source.activeTiles();
    this.container.visible = view.band === 'city' || this.enabled;
    const ratios = this.ratios;
    if (!this.container.visible || ratios === undefined) {
      return;
    }
    if (view.band !== this.band) {
      this.clear();
      this.band = view.band;
    }
    this.buildSome(view, ratios);
    this.showOnly(new Set(view.keys));
  }

  private buildSome(view: ActiveTiles, ratios: Float32Array): void {
    const stale = view.keys.filter((key) => !this.tiles.has(key) || this.dirty.has(key));
    for (const key of stale.slice(0, BUILDS_PER_FRAME)) {
      this.build(key, view, ratios);
    }
  }

  private build(key: string, view: ActiveTiles, ratios: Float32Array): void {
    this.dirty.delete(key);
    this.tiles.get(key)?.destroy();
    this.tiles.delete(key);
    const entry = view.entry(key);
    if (entry === undefined) {
      return;
    }
    const graphics = drawTile(entry, view, ratios, this.roads);
    this.container.addChild(graphics);
    this.tiles.set(key, graphics);
  }

  private showOnly(keys: ReadonlySet<string>): void {
    for (const [key, graphics] of this.tiles) {
      graphics.visible = keys.has(key);
      if (!graphics.visible && this.tiles.size > MAX_TILES) {
        graphics.destroy();
        this.tiles.delete(key);
      }
    }
  }

  private clear(): void {
    for (const graphics of this.tiles.values()) {
      graphics.destroy();
    }
    this.tiles.clear();
    this.dirty.clear();
  }
}
