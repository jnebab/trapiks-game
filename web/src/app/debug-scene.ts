import { createApp, type DebugApp } from '../render/app';
import { createMapView, type MapView } from '../render/map-view';
import { createVehicleLayer } from '../render/vehicle-layer';
import { createVehicleTexture } from '../render/vehicle-texture';
import { createDebugOverlay, type DebugOverlay } from '../hud/debug-overlay';
import { el } from '../hud/dom';
import { createSpeedControl } from '../hud/speed-control';
import type { SimConfig } from '../generated/SimConfig';
import type { ReadyMessage, SnapshotMessage } from '../sim/protocol';
import { startSim, type SimClient } from '../sim/client';
import { SnapshotHistory, createFrame } from '../sim/interpolation';
import { demandFromQuery } from './demand-query';

const SEED = 1;

function showMap(scene: DebugApp, overlay: DebugOverlay, ready: ReadyMessage): MapView {
  const view = createMapView(scene, ready);
  overlay.setRoads(ready.meta.road_count);
  scene.app.ticker.add(() => {
    overlay.setTiles(view.tiles.builtCount, view.tiles.visibleCount, view.tiles.activeBand);
  });
  return view;
}

function showVehicles(scene: DebugApp, view: MapView, history: SnapshotHistory): void {
  const layer = createVehicleLayer(createVehicleTexture(scene.app.renderer));
  view.layers.vehicles.addChild(layer.container);
  const frame = createFrame();
  scene.app.ticker.add(() => {
    history.sample(performance.now(), frame);
    layer.draw(frame);
  });
}

function showError(root: HTMLElement, message: string): void {
  const chip = el('div', 'chip error', message);
  root.appendChild(chip);
}

function receiveSnapshot(
  client: SimClient,
  history: SnapshotHistory,
  overlay: DebugOverlay,
  message: SnapshotMessage,
): void {
  overlay.setVehicles(message.count, message.tick);
  const evicted = history.push(message, performance.now());
  if (evicted) {
    client.returnBuffers(evicted.buffers);
  }
}

export async function runDebugScene(root: HTMLElement, mapName: string): Promise<void> {
  const scene = await createApp(root);
  const overlay = createDebugOverlay(scene.app.ticker, mapName);
  root.appendChild(overlay.element);
  const url = new URL(`maps/${mapName}.bin.gz`, document.baseURI).href;
  const config: SimConfig = {
    seed: SEED,
    mode: 'City',
    vehicles_per_hour: demandFromQuery(window.location.search),
  };
  const history = new SnapshotHistory();
  const client: SimClient = startSim(url, config, {
    onReady: (ready) => {
      const view = showMap(scene, overlay, ready);
      showVehicles(scene, view, history);
    },
    onSnapshot: (message) => {
      receiveSnapshot(client, history, overlay, message);
    },
    onStats: () => undefined,
    onError: (message) => {
      showError(root, message);
    },
  });
  const speed = createSpeedControl(overlay.element, client.setSpeed);
  root.appendChild(speed.element);
}
