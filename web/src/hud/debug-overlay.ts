import type { Ticker } from 'pixi.js';
import { el } from './dom';

const REFRESH_MS = 500;

export interface DebugOverlay {
  element: HTMLElement;
  setRoads: (count: number) => void;
  setVehicles: (count: number, tick: number) => void;
  setTiles: (built: number, visible: number, band: string) => void;
  setMarkings: (built: number) => void;
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

export function createDebugOverlay(ticker: Ticker, mapName: string): DebugOverlay {
  const element = el('div', 'chip');
  element.id = 'debug';
  const state: OverlayState = { roads: 0, vehicles: 0, tick: 0, built: 0, visible: 0 };
  const render = (): void => {
    const fps = Math.round(ticker.FPS);
    element.textContent = overlayText(fps, state, mapName);
    element.dataset.fps = String(fps);
  };
  everyRefresh(ticker, render);
  const setRoads = (count: number): void => {
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
    Object.assign(element.dataset, {
      tilesBuilt: String(built),
      tilesVisible: String(visible),
      band,
    });
  };
  const setMarkings = (built: number): void => {
    element.dataset.markingsBuilt = String(built);
  };
  render();
  return { element, setRoads, setVehicles, setTiles, setMarkings };
}
