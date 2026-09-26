import { el } from './dom';

const TOAST_MS = 2500;

export function showToast(root: HTMLElement, message: string): void {
  const chip = el('div', 'chip error toast', message);
  root.appendChild(chip);
  window.setTimeout(() => {
    chip.remove();
  }, TOAST_MS);
}
