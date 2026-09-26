import { chipButton } from './inspector/action-button';
import { el } from './dom';

export type ToolName = 'select' | 'road';

export interface Toolbar {
  element: HTMLElement;
  setActive: (tool: ToolName) => void;
}

const TOOLS: readonly { tool: ToolName; label: string; key: string }[] = [
  { tool: 'select', label: 'Select', key: 'S' },
  { tool: 'road', label: 'Road', key: 'R' },
];

export function createToolbar(pick: (tool: ToolName) => void): Toolbar {
  const element = el('div', 'toolbar');
  element.setAttribute('role', 'toolbar');
  const buttons = TOOLS.map(({ tool, label, key }) => {
    const button = chipButton(label);
    button.id = `tool-${tool}`;
    button.title = `${label} (${key})`;
    button.addEventListener('click', () => {
      pick(tool);
    });
    element.appendChild(button);
    return { tool, button };
  });
  const setActive = (active: ToolName): void => {
    for (const { tool, button } of buttons) {
      button.setAttribute('aria-pressed', String(tool === active));
    }
  };
  setActive('select');
  return { element, setActive };
}
