import type { StatsSnapshot } from '../../generated/StatsSnapshot';
import { el } from '../../hud/dom';

const KMH_PER_MS = 3.6;

export interface SandboxStats {
  element: HTMLElement;
  set: (stats: StatsSnapshot) => void;
}

function stat(id: string): HTMLElement {
  const element = el('span', 'chip stat', '–');
  element.id = id;
  return element;
}

export function createSandboxStats(): SandboxStats {
  const element = el('div', 'stats-row');
  const active = stat('stat-active');
  const speed = stat('stat-speed');
  const arrivals = stat('stat-arrivals');
  element.append(active, speed, arrivals);
  return {
    element,
    set: (stats) => {
      active.textContent = `${stats.active.toLocaleString('en-US')} vehicles`;
      speed.textContent = `${(stats.mean_speed * KMH_PER_MS).toFixed(0)} km/h`;
      const perHour = Math.round(stats.throughput_per_hour);
      arrivals.textContent = `${perHour.toLocaleString('en-US')} arrivals/h`;
    },
  };
}
