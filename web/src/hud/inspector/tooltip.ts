import { el } from '../dom';

const CURSOR_OFFSET = 14;

export interface Tooltip {
  element: HTMLElement;
  show: (text: string, x: number, y: number) => void;
  move: (x: number, y: number) => void;
  hide: () => void;
}

function clamp(value: number, max: number): number {
  return Math.max(0, Math.min(value, max));
}

export function createTooltip(root: HTMLElement): Tooltip {
  const element = el('div', 'chip tooltip');
  element.setAttribute('role', 'tooltip');
  element.hidden = true;
  root.appendChild(element);
  const move = (x: number, y: number): void => {
    const maxX = window.innerWidth - element.offsetWidth;
    const maxY = window.innerHeight - element.offsetHeight;
    element.style.left = `${String(clamp(x + CURSOR_OFFSET, maxX))}px`;
    element.style.top = `${String(clamp(y + CURSOR_OFFSET, maxY))}px`;
  };
  return {
    element,
    show: (text, x, y) => {
      element.textContent = text;
      element.hidden = false;
      move(x, y);
    },
    move,
    hide: () => {
      element.hidden = true;
    },
  };
}
