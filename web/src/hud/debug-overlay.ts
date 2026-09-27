import type { Ticker } from 'pixi.js';
import { el } from './dom';
import { FrameStats, perfEnabled } from './frame-stats';
import { StepStats, perfFrom } from './step-stats';

const REFRESH_MS = 500;

export interface DebugOverlay {
  element: HTMLElement;
  setRoads: (count: number) => void;
  setVehicles: (count: number, tick: number) => void;
  setTiles: (built: number, visible: number, band: string) => void;
  setMarkings: (built: number, buildings: number) => void;
  setShadowPieces: (count: number) => void;
  setSignalPills: (count: number) => void;
  countSignalUpdate: () => void;
  setTraffic: (visible: boolean) => void;
  setVehicleLayers: (summary: string) => void;
  setLevelView: (view: string) => void;
  setConnections: (summary: string) => void;
  recordSteps: (tick: number, stepMs: Float64Array) => void;
}

interface OverlayState {
  roads: number;
  vehicles: number;
  tick: number;
  built: number;
  visible: number;
}

function overlayText(fps: number, state: OverlayState, mapName: string): string {
  return [
    `FPS ${String(fps)}`,
    `roads ${String(state.roads)}`,
    `vehicles ${String(state.vehicles)}`,
    `tick ${String(state.tick)}`,
    `tiles ${String(state.built)}/${String(state.visible)}`,
    `map ${mapName}`,
  ].join(' · ');
}

function everyRefresh(ticker: Ticker, render: () => void): void {
  let elapsed = 0;
  ticker.add((t) => {
    elapsed += t.deltaMS;
    if (elapsed < REFRESH_MS) {
      return;
    }
    elapsed = 0;
    render();
  });
}

type DepthSetters = Pick<
  DebugOverlay,
  | 'setMarkings'
  | 'setShadowPieces'
  | 'setSignalPills'
  | 'countSignalUpdate'
  | 'setTraffic'
  | 'setVehicleLayers'
  | 'setLevelView'
  | 'setConnections'
>;

function depthSetters(element: HTMLElement): DepthSetters {
  let signalUpdates = 0;
  return {
    setMarkings: (built, buildings) => {
      element.dataset.markingsBuilt = String(built);
      element.dataset.buildingsBuilt = String(buildings);
    },
    setShadowPieces: (count) => {
      element.dataset.shadowPieces = String(count);
    },
    setSignalPills: (count) => {
      element.dataset.signalPills = String(count);
    },
    setTraffic: (visible) => {
      element.dataset.traffic = visible ? '1' : '0';
    },
    setVehicleLayers: (summary) => {
      element.dataset.vehicleLayers = summary;
    },
    setLevelView: (view) => {
      element.dataset.levelView = view;
    },
    setConnections: (summary) => {
      element.dataset.connections = summary;
    },
    countSignalUpdate: () => {
      signalUpdates += 1;
      element.dataset.signalUpdates = String(signalUpdates);
    },
  };
}

function trackFrames(ticker: Ticker, element: HTMLElement): void {
  if (!perfEnabled(window.location.search)) {
    return;
  }
  const stats = new FrameStats();
  ticker.add((t) => {
    stats.record(t.deltaMS);
  });
  everyRefresh(ticker, () => {
    element.dataset.frameP50 = stats.percentile(0.5).toFixed(2);
    element.dataset.frameP95 = stats.percentile(0.95).toFixed(2);
  });
}

function stepRecorder(element: HTMLElement): DebugOverlay['recordSteps'] {
  if (!perfEnabled(window.location.search)) {
    return () => undefined;
  }
  const steps = new StepStats(perfFrom(window.location.search));
  return (tick, stepMs) => {
    steps.record(tick, stepMs);
    Object.assign(element.dataset, steps.summary());
  };
}

function debugVisible(search: string): boolean {
  const params = new URLSearchParams(search);
  return params.get('debug') === '1' || perfEnabled(search);
}

function markLoad(element: HTMLElement, key: 'readyMs' | 'firstTilesMs'): void {
  element.dataset[key] ??= performance.now().toFixed(0);
}

export function createDebugOverlay(ticker: Ticker, mapName: string): DebugOverlay {
  const element = el('div', 'chip');
  element.id = 'debug';
  element.classList.toggle('debug-hidden', !debugVisible(window.location.search));
  const state: OverlayState = { roads: 0, vehicles: 0, tick: 0, built: 0, visible: 0 };
  const render = (): void => {
    const fps = Math.round(ticker.FPS);
    element.textContent = overlayText(fps, state, mapName);
    element.dataset.fps = String(fps);
  };
  everyRefresh(ticker, render);
  trackFrames(ticker, element);
  const setRoads = (count: number): void => {
    markLoad(element, 'readyMs');
    state.roads = count;
    element.dataset.roads = String(count);
    render();
  };
  const setVehicles = (count: number, tick: number): void => {
    state.vehicles = count;
    state.tick = tick;
    element.dataset.vehicles = String(count);
    element.dataset.tick = String(tick);
  };
  const setTiles = (built: number, visible: number, band: string): void => {
    Object.assign(state, { built, visible });
    if (built > 0 && built === visible) {
      markLoad(element, 'firstTilesMs');
    }
    Object.assign(element.dataset, {
      tilesBuilt: String(built),
      tilesVisible: String(visible),
      band,
    });
  };
  render();
  const recordSteps = stepRecorder(element);
  return { element, setRoads, setVehicles, setTiles, recordSteps, ...depthSetters(element) };
}
