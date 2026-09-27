import { el } from '../../hud/dom';
import { routeHash } from '../router';

const STORAGE_KEY = 'trapiks.challengeHintSeen';
const HINT_TEXT =
  'Find the jam, click a junction or road to fix it, then press Evaluate. Full guide: ';

function wasSeen(): boolean {
  try {
    return window.localStorage.getItem(STORAGE_KEY) === '1';
  } catch {
    return false;
  }
}

function markSeen(): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, '1');
  } catch {
    return;
  }
}

function guideLink(): HTMLAnchorElement {
  const link = el('a', 'hint-link', 'How to play');
  link.href = routeHash({ name: 'guide' });
  return link;
}

export function createFirstHint(): HTMLElement | undefined {
  if (wasSeen()) {
    return undefined;
  }
  const element = el('div', 'chip first-hint');
  element.id = 'first-hint';
  const text = el('p', 'hint-text', HINT_TEXT);
  text.appendChild(guideLink());
  const close = el('button', 'hint-close', '×');
  close.type = 'button';
  close.setAttribute('aria-label', 'Dismiss hint');
  close.addEventListener('click', () => {
    markSeen();
    element.remove();
  });
  element.append(text, close);
  return element;
}
