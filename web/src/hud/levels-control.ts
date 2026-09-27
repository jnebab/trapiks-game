import { LEVEL_VIEWS, nextLevelView, parseLevelView, type LevelView } from '../render/level-view';
import { el } from './dom';
import { isEditableTarget } from './keys';

const STORAGE_KEY = 'trapiks.levelView';

const LABELS: Readonly<Record<LevelView, string>> = {
  all: 'All',
  'see-through': 'See-through',
  ground: 'Ground',
  elevated: 'Elevated',
};

function loadView(): LevelView {
  try {
    return parseLevelView(window.localStorage.getItem(STORAGE_KEY));
  } catch {
    return 'all';
  }
}

function storeView(view: LevelView): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, view);
  } catch {
    return;
  }
}

function isCycleKey(event: KeyboardEvent): boolean {
  const plain = !event.ctrlKey && !event.metaKey && !event.altKey;
  return plain && event.key.toLowerCase() === 'l' && !isEditableTarget(event);
}

function optionButton(view: LevelView, choose: (view: LevelView) => void): HTMLButtonElement {
  const button = el('button', 'level-option', LABELS[view]);
  button.type = 'button';
  button.dataset.level = view;
  button.addEventListener('click', () => {
    choose(view);
  });
  return button;
}

export function createLevelsControl(
  apply: (view: LevelView) => void,
  signal: AbortSignal,
): HTMLElement {
  const element = el('div', 'chip levels');
  element.id = 'levels';
  element.setAttribute('role', 'group');
  element.title = 'Road levels (L)';
  element.appendChild(el('span', 'levels-label', 'Levels'));
  let current = loadView();
  const choose = (view: LevelView): void => {
    current = view;
    storeView(view);
    element.dataset.level = view;
    for (const button of buttons) {
      button.setAttribute('aria-pressed', String(button.dataset.level === view));
    }
    apply(view);
  };
  const buttons = LEVEL_VIEWS.map((view) => optionButton(view, choose));
  element.append(...buttons);
  window.addEventListener(
    'keydown',
    (event) => {
      if (isCycleKey(event)) {
        choose(nextLevelView(current));
      }
    },
    { signal },
  );
  choose(current);
  return element;
}
