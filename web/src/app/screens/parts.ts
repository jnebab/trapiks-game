import { el } from '../../hud/dom';
import type { GameContext } from '../screen';
import type { Route } from '../router';

export function button(text: string, className: string, onClick: () => void): HTMLButtonElement {
  const element = el('button', className, text);
  element.type = 'button';
  element.addEventListener('click', onClick);
  return element;
}

export interface TopBar {
  element: HTMLElement;
  slot: HTMLElement;
}

export interface TopBarOptions {
  title: string;
  back: Route;
  extras: HTMLElement[];
}

function helpChip(ctx: GameContext): HTMLButtonElement {
  const help = button('?', 'chip help-chip', () => {
    ctx.navigate({ name: 'guide' });
  });
  help.id = 'help';
  help.title = 'How to play';
  help.setAttribute('aria-label', 'How to play');
  return help;
}

export function createTopBar(ctx: GameContext, options: TopBarOptions): TopBar {
  const element = el('div', 'top-bar');
  const back = button('← Back', 'chip', () => {
    ctx.navigate(options.back);
  });
  back.id = 'back';
  const slot = el('div', 'chip-group');
  const title = el('div', 'chip screen-title', options.title);
  element.append(back, title, slot, ...options.extras, helpChip(ctx), ctx.speed);
  return { element, slot };
}

export function screenRoot(className: string): HTMLElement {
  return el('div', `screen ${className}`);
}

export function starText(stars: number): string {
  const filled = Math.max(0, Math.min(3, Math.round(stars)));
  return '★'.repeat(filled) + '☆'.repeat(3 - filled);
}

export function percent(fraction: number, digits = 0): string {
  return `${(fraction * 100).toFixed(digits)} %`;
}
