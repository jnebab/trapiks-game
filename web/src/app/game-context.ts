import { applyCamera, flyCamera } from '../render/camera-input';
import { fitBounds } from '../render/camera';
import { showToast } from '../hud/toast';
import type { Challenge } from '../generated/Challenge';
import type { SimClient } from '../sim/client';
import type { Region } from '../sim/protocol';
import { attributionText } from './attribution';
import { loadChallenges } from './challenge-list';
import { DEFAULT_VPH, demandFromQuery } from './demand-query';
import { hasCameraQuery, navigate } from './router';
import type { GameContext } from './screen';
import { dropScene, type Shell } from './shell';

const REGION_FIT = 1.15;
const FIT_MARGIN = 24;

type SessionActions = Pick<GameContext, 'startSandbox' | 'startChallenge' | 'showBackground'>;
type CameraActions = Pick<GameContext, 'fitCity' | 'panBy' | 'flyToRegion'>;

function sessionActions(shell: Shell, client: SimClient): SessionActions {
  const { state } = shell;
  const begin = (background: boolean): void => {
    shell.saves.flush();
    dropScene(shell);
    state.background = background;
  };
  return {
    startSandbox: (vehiclesPerHour, log) => {
      begin(false);
      client.startSandbox(vehiclesPerHour, log);
    },
    startChallenge: (challenge, log) => {
      begin(false);
      client.startChallenge(challenge, log);
    },
    showBackground: () => {
      if (!state.background) {
        begin(true);
        client.startSandbox(demandFromQuery(window.location.search) ?? DEFAULT_VPH);
      }
    },
  };
}

function regionCamera(shell: Shell, region: Region) {
  const { width, height } = shell.scene.app.screen;
  const scale = Math.min(width, height) / (2 * region.radius * REGION_FIT);
  return { x: region.x, y: region.y, scale };
}

function cameraActions(shell: Shell): CameraActions {
  const { camera, scene } = shell;
  const apply = (): void => {
    applyCamera(scene.world, camera.camera);
  };
  return {
    fitCity: () => {
      const { width, height } = scene.app.screen;
      camera.camera = fitBounds(shell.state.bounds, width, height, FIT_MARGIN);
      camera.motion = { kind: 'none' };
      apply();
    },
    panBy: (dx) => {
      camera.camera = { ...camera.camera, x: camera.camera.x + dx };
      apply();
    },
    flyToRegion: (region) => {
      const { width, height } = scene.app.screen;
      flyCamera(camera, regionCamera(shell, region), { w: width, h: height });
    },
  };
}

function challengeSource(mapName: string): () => Promise<Challenge[]> {
  let challenges: Promise<Challenge[]> | undefined;
  return () => {
    challenges ??= loadChallenges(mapName);
    return challenges;
  };
}

export function createGameContext(shell: Shell, client: SimClient, mapHash: string): GameContext {
  const search = window.location.search;
  return {
    root: shell.root,
    client,
    mapName: shell.mapName,
    mapHash,
    saves: shell.saves,
    speed: shell.speed,
    camera: shell.camera,
    cameraQuery: hasCameraQuery(search),
    demandQuery: demandFromQuery(search),
    attribution: attributionText(shell.mapName),
    challenges: challengeSource(shell.mapName),
    navigate,
    setSaveTarget: (target) => {
      shell.state.saveTarget = target;
    },
    toast: (text) => {
      showToast(shell.root, text);
    },
    mapScene: () => shell.state.mapScene,
    ...sessionActions(shell, client),
    ...cameraActions(shell),
  };
}
