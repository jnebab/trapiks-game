import { Container } from 'pixi.js';
import type { ReadyMessage } from '../sim/protocol';
import type { DebugApp } from './app';
import { drawAreas } from './areas';
import { fitBounds, type Camera } from './camera';
import { cameraFromQuery } from './camera-query';
import { DetailStore } from './detail-store';
import { createLayers, type Layers } from './layers';
import { NodeStore } from './node-store';
import { RoadStore } from './road-store';
import { TileManager } from './tiles/tile-manager';

const FIT_MARGIN = 24;

export interface MapView {
  root: Container;
  layers: Layers;
  tiles: TileManager;
  roads: RoadStore;
  nodes: NodeStore;
  detail: DetailStore;
}

export function fitCamera(scene: DebugApp, ready: ReadyMessage): Camera {
  const { width, height } = scene.app.screen;
  return fitBounds(ready.meta.bounds, width, height, FIT_MARGIN);
}

export function initialCamera(scene: DebugApp, ready: ReadyMessage): Camera {
  const view = { w: scene.app.screen.width, h: scene.app.screen.height };
  const fit = fitCamera(scene, ready);
  return cameraFromQuery(window.location.search, ready.meta.bounds, view, fit);
}

export function createMapView(scene: DebugApp, ready: ReadyMessage): MapView {
  const root = new Container();
  scene.world.addChild(root);
  const layers = createLayers(root);
  drawAreas(ready.areas, ready.meta.area_kind_names, layers.areas);
  const roads = RoadStore.fromArrays(ready.roads);
  const nodes = NodeStore.fromArrays(ready.nodes);
  const detail = DetailStore.fromArrays(ready.roadSetbacks, ready.junctions, ready.markers);
  const tiles = new TileManager(roads, ready.meta, layers, { detail, nodes });
  return { root, layers, tiles, roads, nodes, detail };
}
