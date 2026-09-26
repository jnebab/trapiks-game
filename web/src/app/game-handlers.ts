import { applyCamera } from '../render/camera-input';
import { initialCamera } from '../render/map-view';
import { el } from '../hud/dom';
import { showToast } from '../hud/toast';
import type { EditCommand } from '../generated/EditCommand';
import type { SimClient, SimHandlers } from '../sim/client';
import type { ReadyMessage } from '../sim/protocol';
import { createGameContext } from './game-context';
import { buildMapScene } from './map-scene';
import { currentRoute, onRouteChange, type Route } from './router';
import type { GameContext, Screen } from './screen';
import { createScreen } from './screens/factory';
import { dropScene, type Shell } from './shell';

function writeSave(shell: Shell, log: EditCommand[]): void {
  const { saveTarget, mapHash } = shell.state;
  if (saveTarget === undefined || mapHash === undefined) {
    return;
  }
  shell.saves.schedule({ ...saveTarget, mapHash, log, updatedAt: Date.now() });
}

function renderRoute(shell: Shell, ctx: GameContext, route: Route): void {
  const { state } = shell;
  shell.saves.flush();
  state.saveTarget = undefined;
  state.screen?.dispose?.();
  state.screen?.element.remove();
  const screen = createScreen(ctx, route);
  state.screen = screen;
  shell.root.appendChild(screen.element);
}

function showScene(shell: Shell, client: SimClient, ready: ReadyMessage): void {
  dropScene(shell);
  const deps = {
    scene: shell.scene,
    camera: shell.camera,
    client,
    overlay: shell.overlay,
    vehicleTexture: shell.vehicleTexture,
    editHost: shell.state.screen?.editHost,
  };
  shell.state.mapScene = buildMapScene(deps, ready);
  shell.state.screen?.onReady?.(ready);
}

function firstReady(shell: Shell, client: SimClient, ready: ReadyMessage): void {
  shell.state.mapHash = ready.meta.map_hash;
  shell.state.bounds = ready.meta.bounds;
  shell.camera.camera = initialCamera(shell.scene, ready);
  applyCamera(shell.scene.world, shell.camera.camera);
  const ctx = createGameContext(shell, client, ready.meta.map_hash);
  onRouteChange((route) => {
    renderRoute(shell, ctx, route);
  });
  renderRoute(shell, ctx, currentRoute());
}

type SceneHandlers = Pick<
  SimHandlers,
  'onReady' | 'onCommandResults' | 'onSnapshot' | 'onStats' | 'onSignals'
>;

function sceneHandlers(shell: Shell, client: () => SimClient): SceneHandlers {
  const { state } = shell;
  return {
    onReady: (ready) => {
      if (state.mapHash === undefined) {
        firstReady(shell, client(), ready);
      }
      if (ready.session === client().session) {
        showScene(shell, client(), ready);
      }
    },
    onCommandResults: (message) => {
      state.mapScene?.onCommandResults(message);
      writeSave(shell, message.log);
      state.screen?.onCommandResults?.(message);
    },
    onSnapshot: (message) => {
      if (state.mapScene === undefined) {
        client().returnBuffers(message.buffers);
        return;
      }
      state.mapScene.onSnapshot(message);
    },
    onStats: (message) => {
      state.mapScene?.onStats(message);
      state.screen?.onStats?.(message);
    },
    onSignals: (message) => {
      state.mapScene?.onSignals(message);
    },
  };
}

export function gameHandlers(shell: Shell, client: () => SimClient): SimHandlers {
  const screen = (): Screen | undefined => shell.state.screen;
  return {
    ...sceneHandlers(shell, client),
    onRunProgress: (message) => screen()?.onRunProgress?.(message),
    onBaseline: (message) => screen()?.onBaseline?.(message),
    onEvaluation: (message) => screen()?.onEvaluation?.(message),
    onNotice: (text) => {
      showToast(shell.root, text);
    },
    onError: (message) => {
      shell.root.appendChild(el('div', 'chip error', message));
    },
  };
}
