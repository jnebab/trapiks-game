import type { Texture } from 'pixi.js';
import type { DebugApp } from '../render/app';
import type { Bounds } from '../render/camera';
import type { CameraState } from '../render/camera-input';
import type { DebugOverlay } from '../hud/debug-overlay';
import type { MapScene } from './map-scene';
import type { SaveStore } from './saves';
import type { SaveTarget, Screen } from './screen';

export interface GameState {
  mapHash: string | undefined;
  bounds: Bounds;
  screen: Screen | undefined;
  mapScene: MapScene | undefined;
  saveTarget: SaveTarget | undefined;
  background: boolean;
}

export interface Shell {
  root: HTMLElement;
  scene: DebugApp;
  overlay: DebugOverlay;
  camera: CameraState;
  vehicleTexture: Texture;
  saves: SaveStore;
  mapName: string;
  speed: HTMLElement;
  state: GameState;
}

export function dropScene(shell: Shell): void {
  shell.state.mapScene?.dispose();
  shell.state.mapScene = undefined;
}
