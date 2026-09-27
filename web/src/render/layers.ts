import { Container } from 'pixi.js';
import { layerIndex, MAX_LAYER, MIN_LAYER } from './layer-range';
import { createShadowContainer, type ShadowContainer } from './shadow-container';

export type RoadPass = 'shadow' | 'outline' | 'fill' | 'markings';

const TUNNEL_ALPHA = 0.4;

export interface Layers {
  areas: Container;
  buildings: Container;
  traffic: Container;
  selection: Container;
  vehicles: (layer: number) => Container;
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
  vehicles: Container;
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
    vehicles: addChild(world),
  };
  if (layer < 0) {
    pair.outline.alpha = TUNNEL_ALPHA;
    pair.fill.alpha = TUNNEL_ALPHA;
    pair.markings.alpha = TUNNEL_ALPHA;
    pair.vehicles.alpha = TUNNEL_ALPHA;
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

function addRoadLayers(world: Container, buildings: Container): RoadPair[] {
  const pairs: RoadPair[] = [];
  for (let layer = MIN_LAYER; layer <= MAX_LAYER; layer += 1) {
    if (layer === 0) {
      world.addChild(buildings);
    }
    pairs.push(addRoadPair(world, layer));
  }
  return pairs;
}

export function createLayers(world: Container): Layers {
  const areas = addChild(world);
  const buildings = new Container();
  const pairs = addRoadLayers(world, buildings);
  const traffic = addChild(world);
  const selection = addChild(world);
  const overlay = addChild(world);
  const pairAt = (layer: number): RoadPair => {
    const pair = pairs[layerIndex(layer)];
    if (pair === undefined) {
      throw new Error('Missing road layer');
    }
    return pair;
  };
  const road = (layer: number, pass: RoadPass): Container => passContainer(pairAt(layer), pass);
  const vehicles = (layer: number): Container => pairAt(layer).vehicles;
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
  return {
    areas,
    buildings,
    traffic,
    selection,
    vehicles,
    overlay,
    road,
    showMarkings,
    updateShadows,
  };
}
