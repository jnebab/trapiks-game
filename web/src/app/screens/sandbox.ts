import { el } from '../../hud/dom';
import { isEditableTarget } from '../../hud/keys';
import { DEFAULT_VPH } from '../demand-query';
import type { GameContext, SaveTarget, Screen } from '../screen';
import { createDemandSlider } from './demand-slider';
import { button, createTopBar, screenRoot } from './parts';
import { createSandboxStats } from './sandbox-stats';
import { loadSave, logOf } from './save-loading';

const SEED = 1;

interface TrafficToggle {
  element: HTMLButtonElement;
  apply: () => void;
  dispose: () => void;
}

function createTrafficToggle(ctx: GameContext): TrafficToggle {
  let on = false;
  const apply = (): void => {
    element.setAttribute('aria-pressed', String(on));
    const scene = ctx.mapScene();
    if (scene !== undefined) {
      scene.traffic.enabled = on;
    }
  };
  const toggle = (): void => {
    on = !on;
    apply();
  };
  const element = button('Traffic', 'chip', toggle);
  element.id = 'traffic-toggle';
  const onKey = (event: KeyboardEvent): void => {
    if (event.key.toLowerCase() === 't' && !isEditableTarget(event) && !event.ctrlKey) {
      toggle();
    }
  };
  window.addEventListener('keydown', onKey);
  apply();
  return {
    element,
    apply,
    dispose: () => {
      window.removeEventListener('keydown', onKey);
    },
  };
}

function start(ctx: GameContext, target: SaveTarget, fresh: boolean): number {
  if (fresh) {
    ctx.saves.clear(ctx.mapHash, 'sandbox');
  }
  const save = fresh ? undefined : loadSave(ctx, 'sandbox');
  const vehiclesPerHour = ctx.demandQuery ?? save?.vehiclesPerHour ?? DEFAULT_VPH;
  target.vehiclesPerHour = vehiclesPerHour;
  ctx.startSandbox(vehiclesPerHour, logOf(save));
  ctx.setSaveTarget(target);
  return vehiclesPerHour;
}

export function createSandboxScreen(ctx: GameContext): Screen {
  const target: SaveTarget = { mode: 'sandbox', seed: SEED };
  const initial = start(ctx, target, false);
  const element = screenRoot('sandbox-screen');
  const traffic = createTrafficToggle(ctx);
  const slider = createDemandSlider(initial, (vph) => {
    target.vehiclesPerHour = vph;
    ctx.client.setDemand(vph);
  });
  const reset = button('Reset', 'chip', () => {
    slider.set(start(ctx, target, true));
  });
  reset.id = 'reset';
  const bar = createTopBar(ctx, {
    title: 'Sandbox',
    back: { name: 'title' },
    extras: [reset, traffic.element],
  });
  const stats = createSandboxStats();
  const bottom = el('div', 'bottom-left');
  bottom.append(slider.element, stats.element);
  element.append(bar.element, bottom);
  return {
    element,
    editHost: { root: element, slot: bar.slot },
    onReady: () => {
      traffic.apply();
    },
    onStats: (message) => {
      stats.set(message.stats);
    },
    dispose: traffic.dispose,
  };
}
