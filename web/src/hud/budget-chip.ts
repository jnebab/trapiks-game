import type { BudgetState } from '../generated/BudgetState';
import { el } from './dom';

export interface BudgetChip {
  element: HTMLElement;
  set: (budget: BudgetState) => void;
}

function label(budget: BudgetState): string {
  const spent = `₱${budget.spent.toLocaleString('en-US')}`;
  if (budget.limit === null) {
    return `Spent ${spent}`;
  }
  return `${spent} / ₱${budget.limit.toLocaleString('en-US')}`;
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
