import { isEditableTarget } from '../../hud/keys';
import type { RoadOptionsRow } from '../../hud/road-options';
import { createToolbar, type ToolName } from '../../hud/toolbar';
import { el } from '../../hud/dom';
import type { RoadTool } from './road-tool';

export interface ToolSwitchDeps {
  root: HTMLElement;
  options: RoadOptionsRow;
  road: RoadTool;
  setSelectEnabled: (enabled: boolean) => void;
  signal: AbortSignal;
}

export interface ToolSwitch {
  current: () => ToolName;
  setLocked: (locked: boolean) => void;
}

const KEY_TOOLS: Readonly<Record<string, ToolName>> = { r: 'road', s: 'select' };

function hasModifier(event: KeyboardEvent): boolean {
  return event.ctrlKey || event.metaKey || event.altKey;
}

export function createToolSwitch(deps: ToolSwitchDeps): ToolSwitch {
  let tool: ToolName = 'select';
  let locked = false;
  const pick = (next: ToolName): void => {
    tool = locked ? 'select' : next;
    toolbar.setActive(tool);
    deps.options.element.hidden = tool !== 'road';
    deps.road.setActive(tool === 'road');
    deps.setSelectEnabled(tool === 'select');
  };
  const toolbar = createToolbar(pick);
  const dock = el('div', 'bottom-centre');
  dock.append(deps.options.element, toolbar.element);
  deps.root.appendChild(dock);
  deps.signal.addEventListener('abort', () => {
    dock.remove();
  });
  const onKey = (event: KeyboardEvent): void => {
    if (isEditableTarget(event) || hasModifier(event)) {
      return;
    }
    const next = KEY_TOOLS[event.key.toLowerCase()];
    if (next !== undefined) {
      pick(next);
    } else if (event.key === 'Escape' && tool === 'road' && !deps.road.cancel()) {
      pick('select');
    }
  };
  window.addEventListener('keydown', onKey, { signal: deps.signal });
  return {
    current: () => tool,
    setLocked: (value) => {
      locked = value;
      deps.road.locked = value;
      if (value) {
        pick('select');
      }
    },
  };
}
