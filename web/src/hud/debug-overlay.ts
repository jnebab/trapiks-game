import type { Ticker } from 'pixi.js';
import { el } from './dom';

const REFRESH_MS = 500;

export interface DebugOverlay {
  element: HTMLElement;
  setRoads: (count: number) => void;
  setVehicles: (count: number, tick: number) => void;
}

export function createDebugOverlay(ticker: Ticker, mapName: string): DebugOverlay {
  const element = el('div', 'chip');
  element.id = 'debug';
  const state = { roads: 0, vehicles: 0, tick: 0 };
  let elapsed = 0;
  const render = (): void => {
    const fps = Math.round(ticker.FPS);
    const parts = [
      `FPS ${String(fps)}`,
      `roads ${String(state.roads)}`,
      `vehicles ${String(state.vehicles)}`,
      `tick ${String(state.tick)}`,
      `map ${mapName}`,
    ];
    element.textContent = parts.join(' · ');
    element.dataset.fps = String(fps);
  };
  ticker.add((t) => {
    elapsed += t.deltaMS;
    if (elapsed < REFRESH_MS) {
      return;
    }
    elapsed = 0;
    render();
  });
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
  render();
  return { element, setRoads, setVehicles };
}
