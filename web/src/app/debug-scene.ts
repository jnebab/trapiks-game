import { drawAreas } from '../render/debug-areas';
import { drawRoads } from '../render/debug-roads';
import { createApp, type DebugApp } from '../render/app';
import { fitBounds } from '../render/camera';
import { applyCamera, wireCameraInput, type CameraState } from '../render/camera-input';
import { createDebugOverlay, type DebugOverlay } from '../hud/debug-overlay';
import { el } from '../hud/dom';
import type { ReadyMessage } from '../sim/protocol';
import { startSim } from '../sim/client';

const FIT_MARGIN = 24;

function showMap(scene: DebugApp, overlay: DebugOverlay, ready: ReadyMessage): void {
  const { app, world } = scene;
  world.addChild(drawAreas(ready.areas, ready.meta.area_kind_names));
  world.addChild(drawRoads(ready.roads));
  const state: CameraState = {
    camera: fitBounds(ready.meta.bounds, app.screen.width, app.screen.height, FIT_MARGIN),
  };
  applyCamera(world, state.camera);
  wireCameraInput(app.canvas, state, world);
  overlay.setRoads(ready.meta.road_count);
}

function showError(root: HTMLElement, message: string): void {
  const chip = el('div', 'chip error', message);
  root.appendChild(chip);
}

export async function runDebugScene(root: HTMLElement, mapName: string): Promise<void> {
  const scene = await createApp(root);
  const overlay = createDebugOverlay(scene.app.ticker, mapName);
  root.appendChild(overlay.element);
  const url = new URL(`maps/${mapName}.bin.gz`, document.baseURI).href;
  startSim(url, {
    onReady: (ready) => {
      showMap(scene, overlay, ready);
    },
    onError: (message) => {
      showError(root, message);
    },
  });
}
