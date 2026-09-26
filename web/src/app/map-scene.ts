import type { Texture } from 'pixi.js';
import type { DebugApp } from '../render/app';
import type { CameraState } from '../render/camera-input';
import { createMapView, type MapView } from '../render/map-view';
import { createRegionBoundary } from '../render/region-boundary';
import { createSignalPills, type SignalPillLayer } from '../render/signal-pills';
import { TrafficLayer } from '../render/traffic-layer';
import { createVehicleLayer } from '../render/vehicle-layer';
import type { DebugOverlay } from '../hud/debug-overlay';
import type { SimClient } from '../sim/client';
import { SnapshotHistory, createFrame } from '../sim/interpolation';
import type {
  CommandResultsMessage,
  ReadyMessage,
  Region,
  SignalsMessage,
  SnapshotMessage,
  StatsMessage,
} from '../sim/protocol';
import { startEditing, type EditHost, type EditSession } from './edit-session';
import { Lifetime } from './lifetime';

export interface SceneDeps {
  scene: DebugApp;
  camera: CameraState;
  client: SimClient;
  overlay: DebugOverlay;
  vehicleTexture: Texture;
  editHost: EditHost | undefined;
}

export interface MapScene {
  view: MapView;
  traffic: TrafficLayer;
  edit: EditSession | undefined;
  onSnapshot: (message: SnapshotMessage) => void;
  onSignals: (message: SignalsMessage) => void;
  onStats: (message: StatsMessage) => void;
  onCommandResults: (message: CommandResultsMessage) => void;
  dispose: () => void;
}

function watchView(deps: SceneDeps, view: MapView, traffic: TrafficLayer, life: Lifetime): void {
  const { scene, camera, overlay } = deps;
  life.onTick(scene.app.ticker, () => {
    view.tiles.update(camera.camera, scene.app.screen.width, scene.app.screen.height);
    traffic.update();
    overlay.setTiles(view.tiles.builtCount, view.tiles.visibleCount, view.tiles.activeBand);
    overlay.setMarkings(view.tiles.markingsBuilt);
    overlay.setShadowPieces(view.tiles.shadowPieces);
    overlay.setTraffic(traffic.visible);
  });
}

function showVehicles(deps: SceneDeps, view: MapView, life: Lifetime): SnapshotHistory {
  const history = new SnapshotHistory();
  const layer = createVehicleLayer(deps.vehicleTexture);
  view.layers.vehicles.addChild(layer.container);
  const frame = createFrame();
  life.onTick(deps.scene.app.ticker, () => {
    history.sample(performance.now(), frame);
    layer.draw(frame);
  });
  return history;
}

function showPills(deps: SceneDeps, view: MapView, ready: ReadyMessage, life: Lifetime) {
  const pills = createSignalPills(deps.scene.app.renderer, ready.signalPills);
  view.layers.overlay.addChild(pills.container);
  life.onTick(deps.scene.app.ticker, () => {
    pills.setScale(deps.camera.camera.scale);
  });
  deps.overlay.setSignalPills(pills.count());
  return pills;
}

function showRegion(deps: SceneDeps, view: MapView, region: Region, life: Lifetime): void {
  const boundary = createRegionBoundary(region);
  view.layers.overlay.addChild(boundary.graphics);
  life.onTick(deps.scene.app.ticker, () => {
    boundary.setScale(deps.camera.camera.scale);
  });
}

function receiveSnapshot(deps: SceneDeps, history: SnapshotHistory, message: SnapshotMessage) {
  deps.overlay.setVehicles(message.count, message.tick);
  const evicted = history.push(message, performance.now());
  if (evicted) {
    deps.client.returnBuffers(evicted.buffers);
  }
}

function editFor(deps: SceneDeps, view: MapView, pills: SignalPillLayer, ready: ReadyMessage) {
  return (life: Lifetime): EditSession | undefined => {
    if (deps.editHost === undefined) {
      return undefined;
    }
    const targets = { view, camera: deps.camera, client: deps.client, pills, meta: ready.meta };
    return startEditing(deps.scene, deps.editHost, targets, life);
  };
}

export function buildMapScene(deps: SceneDeps, ready: ReadyMessage): MapScene {
  const life = new Lifetime();
  const view = createMapView(deps.scene, ready);
  life.onDispose(() => {
    view.root.destroy({ children: true });
  });
  deps.overlay.setRoads(ready.meta.road_count);
  const traffic = new TrafficLayer(view.layers.traffic, view.roads, view.tiles);
  watchView(deps, view, traffic, life);
  const history = showVehicles(deps, view, life);
  const pills = showPills(deps, view, ready, life);
  if (ready.region !== null) {
    showRegion(deps, view, ready.region, life);
  }
  const edit = editFor(deps, view, pills, ready)(life);
  return {
    view,
    traffic,
    edit,
    onSnapshot: (message) => {
      receiveSnapshot(deps, history, message);
    },
    onSignals: (message) => {
      pills.setStates(message.states);
      deps.overlay.countSignalUpdate();
    },
    onStats: (message) => {
      traffic.setRatios(message.roadSpeedRatio);
    },
    onCommandResults: (message) => {
      edit?.onResults(message);
      deps.overlay.setSignalPills(pills.count());
    },
    dispose: () => {
      life.dispose();
    },
  };
}
