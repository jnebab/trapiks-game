import { Container } from 'pixi.js';

export type RoadPass = 'outline' | 'fill' | 'markings';

export const MIN_LAYER = -3;
export const MAX_LAYER = 5;
const TUNNEL_ALPHA = 0.4;

export interface Layers {
  areas: Container;
  vehicles: Container;
  overlay: Container;
  road: (layer: number, pass: RoadPass) => Container;
  showMarkings: (visible: boolean) => void;
}

type RoadPair = Record<RoadPass, Container>;

function addChild(world: Container): Container {
  const container = new Container();
  world.addChild(container);
  return container;
}

function addRoadPair(world: Container, layer: number): RoadPair {
  const pair = { outline: addChild(world), fill: addChild(world), markings: addChild(world) };
  if (layer < 0) {
    pair.outline.alpha = TUNNEL_ALPHA;
    pair.fill.alpha = TUNNEL_ALPHA;
    pair.markings.alpha = TUNNEL_ALPHA;
  }
  pair.markings.visible = false;
  return pair;
}

export function clampLayer(layer: number): number {
  return Math.min(Math.max(layer, MIN_LAYER), MAX_LAYER);
}

export function createLayers(world: Container): Layers {
  const areas = addChild(world);
  const pairs: RoadPair[] = [];
  for (let layer = MIN_LAYER; layer <= MAX_LAYER; layer += 1) {
    pairs.push(addRoadPair(world, layer));
  }
  const vehicles = addChild(world);
  const overlay = addChild(world);
  const road = (layer: number, pass: RoadPass): Container => {
    const pair = pairs[clampLayer(layer) - MIN_LAYER] ?? pairs[0 - MIN_LAYER];
    if (pair === undefined) {
      throw new Error('Missing road layer');
    }
    return pair[pass];
  };
  const showMarkings = (visible: boolean): void => {
    for (const pair of pairs) {
      pair.markings.visible = visible;
    }
  };
  return { areas, vehicles, overlay, road, showMarkings };
}
