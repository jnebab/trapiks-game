import type { BudgetState } from '../generated/BudgetState';
import { el } from './dom';
import { formatPesos } from './money';

export interface BudgetChip {
  element: HTMLElement;
  set: (budget: BudgetState) => void;
}

function label(budget: BudgetState): string {
  const spent = formatPesos(budget.spent);
  if (budget.limit === null) {
    return `Spent ${spent}`;
  }
  return `${spent} / ${formatPesos(budget.limit)}`;
}

export function createBudgetChip(): BudgetChip {
  const element = el('div', 'chip');
  element.id = 'budget';
  const set = (budget: BudgetState): void => {
    element.dataset.spent = String(budget.spent);
    element.textContent = label(budget);
  };
  set({ limit: null, spent: 0 });
  return { element, set };
}
