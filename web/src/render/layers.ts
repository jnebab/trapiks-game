import { Container } from 'pixi.js';
import { createShadowContainer, type ShadowContainer } from './shadow-container';

export type RoadPass = 'shadow' | 'outline' | 'fill' | 'markings';

export const MIN_LAYER = -3;
export const MAX_LAYER = 5;
const TUNNEL_ALPHA = 0.4;

export interface Layers {
  areas: Container;
  traffic: Container;
  selection: Container;
  vehicles: Container;
  overlay: Container;
  road: (layer: number, pass: RoadPass) => Container;
  showMarkings: (visible: boolean) => void;
  updateShadows: (visible: boolean, scale: number) => void;
}

interface RoadPair {
  shadow: ShadowContainer | undefined;
  outline: Container;
  fill: Container;
  markings: Container;
}

function addChild(world: Container): Container {
  const container = new Container();
  world.addChild(container);
  return container;
}

function addShadow(world: Container, layer: number): ShadowContainer | undefined {
  if (layer < 1) {
    return undefined;
  }
  const shadow = createShadowContainer();
  world.addChild(shadow.container);
  return shadow;
}

function addRoadPair(world: Container, layer: number): RoadPair {
  const shadow = addShadow(world, layer);
  const pair = {
    shadow,
    outline: addChild(world),
    fill: addChild(world),
    markings: addChild(world),
  };
  if (layer < 0) {
    pair.outline.alpha = TUNNEL_ALPHA;
    pair.fill.alpha = TUNNEL_ALPHA;
    pair.markings.alpha = TUNNEL_ALPHA;
  }
  pair.markings.visible = false;
  return pair;
}

function passContainer(pair: RoadPair, pass: RoadPass): Container {
  if (pass !== 'shadow') {
    return pair[pass];
  }
  if (pair.shadow === undefined) {
    throw new Error('No shadow container below layer 1');
  }
  return pair.shadow.container;
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
  const traffic = addChild(world);
  const selection = addChild(world);
  const vehicles = addChild(world);
  const overlay = addChild(world);
  const road = (layer: number, pass: RoadPass): Container => {
    const pair = pairs[clampLayer(layer) - MIN_LAYER] ?? pairs[0 - MIN_LAYER];
    if (pair === undefined) {
      throw new Error('Missing road layer');
    }
    return passContainer(pair, pass);
  };
  const showMarkings = (visible: boolean): void => {
    for (const pair of pairs) {
      pair.markings.visible = visible;
    }
  };
  const updateShadows = (visible: boolean, scale: number): void => {
    for (const pair of pairs) {
      pair.shadow?.update(visible, scale);
    }
  };
  return { areas, traffic, selection, vehicles, overlay, road, showMarkings, updateShadows };
}
