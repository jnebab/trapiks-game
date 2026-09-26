import { actionButton, REQUOTE_EVENT, type ActionClient } from './action-button';
import type { Tooltip } from './tooltip';

export interface UndoChip {
  element: HTMLButtonElement;
  refresh: () => void;
}

export function createUndoChip(client: ActionClient, tooltip: Tooltip): UndoChip {
  const element = actionButton('Undo', () => 'Undo', client, tooltip);
  element.id = 'undo';
  const refresh = (): void => {
    element.dispatchEvent(new Event(REQUOTE_EVENT));
  };
  refresh();
  return { element, refresh };
}
