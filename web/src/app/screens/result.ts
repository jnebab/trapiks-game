import type { FailReason } from '../../generated/FailReason';
import type { Score } from '../../generated/Score';
import { el } from '../../hud/dom';
import { formatPesos } from '../../hud/money';
import { button, percent, starText } from './parts';

const REASONS: Readonly<Record<FailReason, string>> = {
  NotEnoughImprovement: 'Delay did not drop enough',
  ThroughputDropped: 'Fewer trips got through',
  OverBudget: 'Over budget',
};

export interface ResultActions {
  keepFixing: () => void;
  next: () => void;
}

function fact(label: string, value: string, id: string): HTMLElement {
  const row = el('div', 'result-fact');
  const number = el('span', 'result-value', value);
  number.id = id;
  row.append(el('span', 'result-label', label), number);
  return row;
}

function facts(score: Score): HTMLElement {
  const list = el('div', 'result-facts');
  const improvement = fact('Delay cut', percent(score.improvement, 1), 'improvement');
  improvement.dataset.value = String(score.improvement);
  list.append(
    improvement,
    fact('Throughput', percent(score.throughput_ratio, 1), 'throughput'),
    fact('Cost', formatPesos(score.cost), 'cost'),
  );
  return list;
}

function reasons(score: Score): HTMLElement {
  const list = el('ul', 'result-reasons');
  for (const reason of score.reasons) {
    list.appendChild(el('li', undefined, REASONS[reason]));
  }
  return list;
}

export function createResultModal(score: Score, actions: ResultActions): HTMLElement {
  const backdrop = el('div', 'modal-backdrop');
  const modal = el('div', 'result-modal');
  modal.id = 'result';
  modal.setAttribute('role', 'dialog');
  modal.dataset.passed = String(score.passed);
  const stars = el('div', 'stars result-stars', starText(score.stars));
  stars.dataset.stars = String(score.stars);
  const close = (action: () => void) => () => {
    backdrop.remove();
    action();
  };
  const buttons = el('div', 'result-buttons');
  buttons.append(
    button('Keep fixing', 'chip', close(actions.keepFixing)),
    button('Next challenge', 'chip primary', close(actions.next)),
  );
  modal.append(
    el('h2', 'result-title', score.passed ? 'Passed' : 'Not yet'),
    stars,
    facts(score),
    reasons(score),
    buttons,
  );
  backdrop.appendChild(modal);
  return backdrop;
}
