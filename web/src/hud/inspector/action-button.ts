import type { EditCommand } from '../../generated/EditCommand';
import type { QuoteOutcome } from '../../generated/QuoteOutcome';
import type { SimClient } from '../../sim/client';
import { el } from '../dom';
import type { Tooltip } from './tooltip';
import { costLabel } from './view-models';

const QUOTE_DEBOUNCE_MS = 80;
export const REQUOTE_EVENT = 'requote';

export type ActionClient = Pick<SimClient, 'quote' | 'sendCommand'>;

export interface ActionLabel {
  text: string;
  action: string;
}

function labelOf(label: string | ActionLabel): ActionLabel {
  return typeof label === 'string' ? { text: label, action: label } : label;
}

function setDisabled(button: HTMLButtonElement, disabled: boolean): void {
  button.setAttribute('aria-disabled', String(disabled));
}

export function isDisabled(button: HTMLElement): boolean {
  return button.getAttribute('aria-disabled') === 'true';
}

export function chipButton(text: string, className = 'chip'): HTMLButtonElement {
  const button = el('button', className, text);
  button.type = 'button';
  return button;
}

interface Hover {
  x: number;
  y: number;
  hovered: boolean;
}

function quoter(
  button: HTMLButtonElement,
  quote: () => Promise<QuoteOutcome>,
  show: (outcome: QuoteOutcome) => void,
) {
  let timer: number | undefined;
  const requote = (): void => {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => {
      void quote().then((outcome) => {
        setDisabled(button, 'Err' in outcome);
        show(outcome);
      });
    }, QUOTE_DEBOUNCE_MS);
  };
  return { requote };
}

function wireHover(button: HTMLButtonElement, tooltip: Tooltip, hover: Hover, requote: () => void) {
  const track = (event: PointerEvent): void => {
    hover.x = event.clientX;
    hover.y = event.clientY;
    tooltip.move(hover.x, hover.y);
  };
  button.addEventListener('pointerenter', (event) => {
    hover.hovered = true;
    track(event);
    document.addEventListener('input', requote);
    requote();
  });
  button.addEventListener('pointermove', track);
  button.addEventListener('pointerleave', () => {
    hover.hovered = false;
    document.removeEventListener('input', requote);
    tooltip.hide();
  });
}

export function actionButton(
  label: string | ActionLabel,
  command: () => EditCommand,
  client: ActionClient,
  tooltip: Tooltip,
): HTMLButtonElement {
  const { text, action } = labelOf(label);
  const button = chipButton(text);
  const hover: Hover = { x: 0, y: 0, hovered: false };
  const { requote } = quoter(
    button,
    () => client.quote(command()),
    (outcome) => {
      if (hover.hovered && button.isConnected) {
        tooltip.show(costLabel(outcome, action), hover.x, hover.y);
      }
    },
  );
  wireHover(button, tooltip, hover, requote);
  button.addEventListener(REQUOTE_EVENT, requote);
  button.addEventListener('click', () => {
    if (!isDisabled(button)) {
      client.sendCommand(command());
    }
  });
  return button;
}
