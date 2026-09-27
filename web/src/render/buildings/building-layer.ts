import { Graphics, type Container } from 'pixi.js';
import type { RoadClass } from '../../generated/RoadClass';
import type { DetailStore } from '../detail-store';
import type { NodeStore } from '../node-store';
import type { Rect } from '../rect';
import type { RoadStore } from '../road-store';
import { drawBuildings } from './draw';
import { LotGrid } from './lot-grid';
import { distanceTo, type Obb } from './obb';
import { placeLots, type Lot, type LotStore } from './placement';

export const MAX_BUILDINGS_PER_TILE = 600;
export const JUNCTION_CLEARANCE = 20;
export const ROAD_PROBE = 2;
const NODE_MARGIN = 80;

export interface BuildingSources {
  roads: RoadStore;
  nodes: NodeStore;
  detail: DetailStore;
  classNames: readonly RoadClass[];
  pickRoad: (x: number, y: number, tolerance: number) => number | undefined;
}

export interface BuildingTile {
  roads: readonly number[];
  bounds: Rect;
}

export function thin(lots: readonly Lot[], cap: number): Lot[] {
  if (lots.length <= cap) {
    return [...lots];
  }
  const every = Math.ceil(lots.length / cap);
  return lots.filter((_, i) => i % every === 0);
}

function inside(bounds: Rect, x: number, y: number, margin: number): boolean {
  return (
    x >= bounds.minX - margin &&
    x <= bounds.maxX + margin &&
    y >= bounds.minY - margin &&
    y <= bounds.maxY + margin
  );
}

function junctionPoints(sources: BuildingSources, bounds: Rect): [number, number][] {
  const points: [number, number][] = [];
  for (const node of sources.detail.junctions.keys()) {
    const x = sources.nodes.x(node);
    const y = sources.nodes.y(node);
    if (inside(bounds, x, y, NODE_MARGIN)) {
      points.push([x, y]);
    }
  }
  return points;
}

function lotStore(sources: BuildingSources, bounds: Rect): LotStore {
  const nodes = junctionPoints(sources, bounds);
  return {
    roads: sources.roads,
    classNames: sources.classNames,
    hitsRoad: (x, y) => sources.pickRoad(x, y, ROAD_PROBE) !== undefined,
    nearJunction: (box: Obb) => nodes.some(([x, y]) => distanceTo(box, x, y) <= JUNCTION_CLEARANCE),
    grid: new LotGrid(),
  };
}

export function tileLots(sources: BuildingSources, tile: BuildingTile): Lot[] {
  const store = lotStore(sources, tile.bounds);
  const ids = [...tile.roads].sort((a, b) => a - b);
  const lots = ids.flatMap((road) => [0, 1].flatMap((side) => placeLots(road, side, store)));
  return thin(lots, MAX_BUILDINGS_PER_TILE);
}

export function buildTileBuildings(
  sources: BuildingSources,
  tile: BuildingTile,
  container: Container,
): Graphics {
  const graphics = new Graphics();
  drawBuildings(tileLots(sources, tile), graphics);
  container.addChild(graphics);
  return graphics;
}
