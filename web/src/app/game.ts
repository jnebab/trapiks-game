import { createApp } from '../render/app';
import { wireCameraInput, type CameraState } from '../render/camera-input';
import { createVehicleTexture } from '../render/vehicle-texture';
import { createDebugOverlay } from '../hud/debug-overlay';
import { createSpeedControl } from '../hud/speed-control';
import type { SimConfig } from '../generated/SimConfig';
import { startSim, type SimClient } from '../sim/client';
import { createAttribution } from './attribution';
import { DEFAULT_VPH, demandFromQuery } from './demand-query';
import { gameHandlers } from './game-handlers';
import { browserSaves } from './saves';
import type { GameState, Shell } from './shell';

function initialConfig(): SimConfig {
  const vehicles = demandFromQuery(window.location.search) ?? DEFAULT_VPH;
  return { seed: 1, mode: 'City', vehicles_per_hour: vehicles };
}

function initialState(): GameState {
  return {
    mapHash: undefined,
    bounds: [0, 0, 1, 1],
    screen: undefined,
    mapScene: undefined,
    saveTarget: undefined,
    background: true,
  };
}

export async function runGame(root: HTMLElement, mapName: string): Promise<void> {
  const scene = await createApp(root);
  const overlay = createDebugOverlay(scene.app.ticker, mapName);
  root.append(overlay.element, createAttribution(mapName));
  const camera: CameraState = { camera: { x: 0, y: 0, scale: 1 }, motion: { kind: 'none' } };
  wireCameraInput(scene.app.canvas, camera, scene.world, scene.app.ticker);
  const holder: { client?: SimClient } = {};
  const speed = createSpeedControl(overlay.element, (value) => {
    holder.client?.setSpeed(value);
  });
  const shell: Shell = {
    root,
    scene,
    overlay,
    camera,
    vehicleTexture: createVehicleTexture(scene.app.renderer),
    saves: browserSaves(),
    mapName,
    speed: speed.element,
    state: initialState(),
  };
  const clientOf = (): SimClient => {
    if (holder.client === undefined) {
      throw new Error('Sim client is not started');
    }
    return holder.client;
  };
  const url = new URL(`maps/${mapName}.bin.gz`, document.baseURI).href;
  holder.client = startSim(url, initialConfig(), gameHandlers(shell, clientOf));
}
