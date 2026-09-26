import { el } from '../../hud/dom';
import type { Challenge } from '../../generated/Challenge';
import type { RunResult } from '../../generated/RunResult';
import { percent } from './parts';

export interface Objective {
  element: HTMLElement;
  measuring: (done: number, total: number) => void;
  measured: (baseline: RunResult) => void;
}

function fraction(done: number, total: number): number {
  return total > 0 ? Math.min(1, done / total) : 0;
}

export function progressText(label: string, done: number, total: number): string {
  return `${label}… ${percent(fraction(done, total))}`;
}

export function createObjective(challenge: Challenge): Objective {
  const element = el('div', 'chip objective');
  element.id = 'objective';
  const target = `Cut delay by ${percent(challenge.target_improvement)}`;
  const measuring = (done: number, total: number): void => {
    element.textContent = progressText('Measuring baseline', done, total);
  };
  measuring(0, 1);
  return {
    element,
    measuring,
    measured: (baseline) => {
      element.textContent = `${target} · baseline ${baseline.mean_delay_s.toFixed(1)} s`;
      element.dataset.baseline = String(baseline.mean_delay_s);
    },
  };
}
