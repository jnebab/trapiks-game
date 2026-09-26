import { el } from '../../hud/dom';
import type { Route } from '../router';
import type { GameContext, Screen } from '../screen';
import type { SaveMode } from '../saves';
import { challengeIdOf } from '../challenge-list';
import { button } from './parts';

const PAN_PX_PER_S = 20;

function routeOf(mode: SaveMode): Route {
  return mode === 'sandbox' ? { name: 'sandbox' } : { name: 'challenge', id: challengeIdOf(mode) };
}

function menu(ctx: GameContext): HTMLElement {
  const buttons = el('div', 'title-buttons');
  buttons.append(
    button('Challenges', 'chip title-button', () => {
      ctx.navigate({ name: 'challenges' });
    }),
    button('Sandbox', 'chip title-button', () => {
      ctx.navigate({ name: 'sandbox' });
    }),
  );
  const latest = ctx.saves.latest(ctx.mapHash);
  if (latest !== undefined) {
    const resume = button('Continue', 'chip title-button primary', () => {
      ctx.navigate(routeOf(latest.mode));
    });
    resume.id = 'continue';
    buttons.prepend(resume);
  }
  return buttons;
}

function startPan(ctx: GameContext): () => void {
  let last = performance.now();
  let frame = 0;
  const step = (now: number): void => {
    ctx.panBy((-PAN_PX_PER_S * (now - last)) / 1000);
    last = now;
    frame = requestAnimationFrame(step);
  };
  frame = requestAnimationFrame(step);
  return () => {
    cancelAnimationFrame(frame);
  };
}

export function createTitleScreen(ctx: GameContext): Screen {
  ctx.showBackground();
  const element = el('div', 'title-screen');
  element.id = 'title';
  const card = el('div', 'title-card');
  card.append(
    el('h1', 'wordmark', 'Trapiks'),
    el('p', 'tagline', "Fix Metro Manila's traffic"),
    menu(ctx),
    el('p', 'credits', ctx.attribution),
  );
  element.appendChild(card);
  if (ctx.cameraQuery) {
    return { element };
  }
  ctx.fitCity();
  const stopPan = startPan(ctx);
  return { element, dispose: stopPan };
}
