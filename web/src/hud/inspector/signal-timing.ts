import type { SignalInfo } from '../../generated/SignalInfo';
import { el } from '../dom';
import { actionButton } from './action-button';
import { section, type ViewContext } from './controls';

const MIN_GREEN_S = 5;
const MAX_GREEN_S = 120;
const CLEARANCE_S = 5;

function slider(min: number, max: number, value: number): HTMLInputElement {
  const input = el('input', 'inspector-slider');
  input.type = 'range';
  input.min = String(min);
  input.max = String(max);
  input.value = String(Math.min(value, max));
  return input;
}

function sliderRow(label: string, input: HTMLInputElement): HTMLElement {
  const value = el('span', 'inspector-value', `${input.value} s`);
  input.setAttribute('aria-label', label);
  input.addEventListener('input', () => {
    value.textContent = `${input.value} s`;
  });
  const element = el('label', 'inspector-row');
  element.append(el('span', 'inspector-label', label), input, value);
  return element;
}

function cycleOf(greens: readonly HTMLInputElement[]): number {
  return greens.reduce((sum, input) => sum + Number(input.value) + CLEARANCE_S, 0);
}

export function signalTiming(node: number, signal: SignalInfo, ctx: ViewContext): HTMLElement {
  const greens = signal.greens_s.map((green) => slider(MIN_GREEN_S, MAX_GREEN_S, green));
  const offset = slider(0, cycleOf(greens) - 1, signal.offset_s);
  const offsetRow = sliderRow('Offset', offset);
  for (const input of greens) {
    input.addEventListener('input', () => {
      offset.max = String(cycleOf(greens) - 1);
      offset.dispatchEvent(new Event('input'));
    });
  }
  const apply = actionButton(
    'Apply timing',
    () => ({
      SetSignalTiming: {
        node,
        greens_s: greens.map((input) => Number(input.value)),
        offset_s: Number(offset.value),
      },
    }),
    ctx.client,
    ctx.tooltip,
  );
  const rows = greens.map((input, phase) => sliderRow(`Phase ${String(phase + 1)}`, input));
  return section('Signal timing', ...rows, offsetRow, apply);
}
