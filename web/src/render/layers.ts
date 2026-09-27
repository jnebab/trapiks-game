import { Container } from 'pixi.js';
import { layerIndex, MAX_LAYER, MIN_LAYER } from './layer-range';
import { buildingsAlpha, levelStyle, type LevelView } from './level-view';
import { createShadowContainer, type ShadowContainer } from './shadow-container';

export type RoadPass = 'shadow' | 'outline' | 'fill' | 'markings';

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
  setLevelView: (view: LevelView) => void;
  levelView: () => LevelView;
}

interface RoadPair {
  layer: number;
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
  return {
    layer,
    shadow,
    outline: addChild(world),
    fill: addChild(world),
    markings: addChild(world),
    vehicles: addChild(world),
  };
}

interface ViewState {
  view: LevelView;
  markings: boolean;
  shadows: boolean;
}

function applyView(pair: RoadPair, state: ViewState): void {
  const style = levelStyle(state.view, pair.layer);
  for (const container of [pair.outline, pair.fill, pair.markings, pair.vehicles]) {
    container.alpha = style.alpha;
    container.visible = style.visible;
  }
  pair.markings.visible = style.visible && state.markings;
  if (pair.shadow !== undefined) {
    pair.shadow.container.visible = style.shadow && state.shadows;
  }
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

type LevelControls = Pick<Layers, 'showMarkings' | 'updateShadows' | 'setLevelView' | 'levelView'>;

function levelControls(pairs: readonly RoadPair[], buildings: Container): LevelControls {
  const state: ViewState = { view: 'all', markings: false, shadows: false };
  const applyAll = (): void => {
    buildings.alpha = buildingsAlpha(state.view);
    for (const pair of pairs) {
      applyView(pair, state);
    }
  };
  applyAll();
  return {
    showMarkings: (visible) => {
      state.markings = visible;
      applyAll();
    },
    updateShadows: (visible, scale) => {
      state.shadows = visible;
      for (const pair of pairs) {
        pair.shadow?.update(visible, scale);
      }
      applyAll();
    },
    setLevelView: (view) => {
      state.view = view;
      applyAll();
    },
    levelView: () => state.view,
  };
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
  return {
    areas,
    buildings,
    traffic,
    selection,
    overlay,
    vehicles: (layer) => pairAt(layer).vehicles,
    road: (layer, pass) => passContainer(pairAt(layer), pass),
    ...levelControls(pairs, buildings),
  };
}
