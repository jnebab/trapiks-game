import type { Challenge } from '../../generated/Challenge';
import { el } from '../../hud/dom';
import { formatPesos } from '../../hud/money';
import { challengeMode } from '../challenge-list';
import type { GameContext, Screen } from '../screen';
import { button, percent, starText } from './parts';

function card(ctx: GameContext, challenge: Challenge): HTMLElement {
  const element = button('', 'challenge-card', () => {
    ctx.navigate({ name: 'challenge', id: challenge.id });
  });
  element.dataset.id = challenge.id;
  const stars = ctx.saves.bestStars(ctx.mapHash, challengeMode(challenge.id)) ?? 0;
  const facts = el('div', 'card-facts');
  facts.append(
    el('span', 'chip', `Budget ${formatPesos(challenge.budget)}`),
    el('span', 'chip', `Target −${percent(challenge.target_improvement)} delay`),
  );
  const rating = el('div', 'stars', starText(stars));
  rating.dataset.stars = String(stars);
  element.append(
    el('h2', 'card-name', challenge.name),
    el('p', 'card-blurb', challenge.blurb),
    facts,
    rating,
  );
  return element;
}

function fill(ctx: GameContext, grid: HTMLElement): void {
  ctx.challenges().then(
    (list) => {
      grid.replaceChildren(...list.map((challenge) => card(ctx, challenge)));
    },
    (error: unknown) => {
      ctx.toast(error instanceof Error ? error.message : 'Challenges could not be loaded');
    },
  );
}

export function createChallengesScreen(ctx: GameContext): Screen {
  ctx.showBackground();
  const element = el('div', 'challenges-screen');
  element.id = 'challenges';
  const header = el('div', 'challenges-header');
  const back = button('← Back', 'chip', () => {
    ctx.navigate({ name: 'title' });
  });
  header.append(back, el('h1', 'challenges-title', 'Challenges'));
  const grid = el('div', 'challenge-grid');
  element.append(header, grid);
  fill(ctx, grid);
  return { element };
}
