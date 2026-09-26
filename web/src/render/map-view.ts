import type { ReadyMessage } from '../sim/protocol';
import type { DebugApp } from './app';
import { drawAreas } from './areas';
import { fitBounds } from './camera';
import { applyCamera, wireCameraInput, type CameraState } from './camera-input';
import { cameraFromQuery } from './camera-query';
import { createLayers, type Layers } from './layers';
import { TileManager } from './tiles/tile-manager';

const FIT_MARGIN = 24;

export interface MapView {
  state: CameraState;
  layers: Layers;
  tiles: TileManager;
}

function initialState(scene: DebugApp, ready: ReadyMessage): CameraState {
  const view = { w: scene.app.screen.width, h: scene.app.screen.height };
  const fit = fitBounds(ready.meta.bounds, view.w, view.h, FIT_MARGIN);
  const camera = cameraFromQuery(window.location.search, ready.meta.bounds, view, fit);
  return { camera, motion: { kind: 'none' } };
}

export function createMapView(scene: DebugApp, ready: ReadyMessage): MapView {
  const { app, world } = scene;
  const layers = createLayers(world);
  drawAreas(ready.areas, ready.meta.area_kind_names, layers.areas);
  const tiles = new TileManager(ready.roads, ready.meta, layers);
  const state = initialState(scene, ready);
  applyCamera(world, state.camera);
  wireCameraInput(app.canvas, state, world, app.ticker);
  app.ticker.add(() => {
    tiles.update(state.camera, app.screen.width, app.screen.height);
  });
  return { state, layers, tiles };
}
