import { el } from './dom';

const CURSOR_OFFSET = 14;

export interface RoadTooltip {
  element: HTMLElement;
  show: (text: string, layer: number, x: number, y: number) => void;
  hide: () => void;
}

export function createRoadTooltip(root: HTMLElement): RoadTooltip {
  const element = el('div', 'road-tooltip');
  element.id = 'road-tooltip';
  element.setAttribute('role', 'tooltip');
  element.hidden = true;
  const level = el('div', 'chip');
  const cost = el('div', 'chip');
  element.append(level, cost);
  root.appendChild(element);
  return {
    element,
    show: (text, layer, x, y) => {
      cost.textContent = text;
      level.textContent = `Elevation: ${String(layer)}`;
      level.hidden = layer <= 0;
      element.style.left = `${String(x + CURSOR_OFFSET)}px`;
      element.style.top = `${String(y + CURSOR_OFFSET)}px`;
      element.hidden = false;
    },
    hide: () => {
      element.hidden = true;
    },
  };
}
