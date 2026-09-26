import type { EditCommand } from '../../generated/EditCommand';
import { el } from '../dom';
import type { ArmStores } from './arm-labels';
import { actionButton, chipButton, type ActionClient } from './action-button';
import type { Tooltip } from './tooltip';

export interface ViewContext {
  client: ActionClient;
  tooltip: Tooltip;
  stores: ArmStores;
}

export interface Step {
  text: string;
  action: string;
  command: EditCommand | null;
}

export function commandButton(step: Step, ctx: ViewContext): HTMLButtonElement {
  const { command } = step;
  if (command === null) {
    const button = chipButton(step.text);
    button.setAttribute('aria-disabled', 'true');
    return button;
  }
  return actionButton(step, () => command, ctx.client, ctx.tooltip);
}

export function row(label: string, ...children: HTMLElement[]): HTMLElement {
  const element = el('div', 'inspector-row');
  element.append(el('span', 'inspector-label', label), ...children);
  return element;
}

export function stepper(
  label: string,
  value: string,
  steps: [Step, Step],
  ctx: ViewContext,
): HTMLElement {
  const [down, up] = steps;
  const stepperRow = row(
    label,
    commandButton(down, ctx),
    el('span', 'inspector-value', value),
    commandButton(up, ctx),
  );
  stepperRow.classList.add('stepper');
  return stepperRow;
}

export function segmented(buttons: HTMLButtonElement[]): HTMLElement {
  const group = el('div', 'segmented');
  group.setAttribute('role', 'group');
  group.append(...buttons);
  return group;
}

export function pressed(button: HTMLButtonElement, isPressed: boolean): HTMLButtonElement {
  button.setAttribute('aria-pressed', String(isPressed));
  return button;
}

export function header(title: string, subtitle: string): HTMLElement {
  const element = el('header', 'inspector-header');
  element.append(el('h2', 'inspector-title', title), el('p', 'inspector-subtitle', subtitle));
  return element;
}

export function section(title: string, ...children: HTMLElement[]): HTMLElement {
  const element = el('section', 'inspector-section');
  element.append(el('h3', 'inspector-section-title', title), ...children);
  return element;
}
