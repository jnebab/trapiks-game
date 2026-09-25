import type { Ticker } from 'pixi.js';
import { el } from './dom';

const REFRESH_MS = 500;

export interface DebugOverlay {
  element: HTMLElement;
  setRoads: (count: number) => void;
}

export function createDebugOverlay(ticker: Ticker, mapName: string): DebugOverlay {
  const element = el('div', 'chip');
  element.id = 'debug';
  let roads = 0;
  let elapsed = 0;
  const render = (): void => {
    const fps = Math.round(ticker.FPS);
    element.textContent = `FPS ${String(fps)} · roads ${String(roads)} · map ${mapName}`;
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
    roads = count;
    element.dataset.roads = String(count);
    render();
  };
  render();
  return { element, setRoads };
}
