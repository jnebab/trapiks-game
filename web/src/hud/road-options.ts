import { chipButton } from './inspector/action-button';
import { el } from './dom';

export const MAX_ELEVATION = 2;

export interface RoadOptions {
  lanes: readonly [number, number];
  layer: number;
  curve: boolean;
}

export interface RoadOptionsRow {
  element: HTMLElement;
  options: RoadOptions;
}

const PRESETS: readonly { label: string; lanes: readonly [number, number] }[] = [
  { label: '1+1', lanes: [1, 1] },
  { label: '2+2', lanes: [2, 2] },
  { label: '3+3', lanes: [3, 3] },
  { label: '1 →', lanes: [1, 0] },
  { label: '2 →', lanes: [2, 0] },
];

function lanePresets(options: RoadOptions, changed: () => void): HTMLElement {
  const group = el('div', 'segmented lane-presets');
  const buttons = PRESETS.map((preset) => {
    const button = chipButton(preset.label);
    button.addEventListener('click', () => {
      options.lanes = preset.lanes;
      buttons.forEach((b) => {
        b.setAttribute('aria-pressed', String(b === button));
      });
      changed();
    });
    button.setAttribute('aria-pressed', String(preset.lanes === options.lanes));
    group.appendChild(button);
    return button;
  });
  return group;
}

function elevation(options: RoadOptions, changed: () => void): HTMLElement {
  const group = el('div', 'chip elevation');
  const value = el('span', 'elevation-value', String(options.layer));
  value.id = 'elevation-value';
  const step = (label: string, name: string, delta: number): HTMLButtonElement => {
    const button = chipButton(label, 'chip stepper');
    button.setAttribute('aria-label', name);
    button.addEventListener('click', () => {
      options.layer = Math.min(MAX_ELEVATION, Math.max(0, options.layer + delta));
      value.textContent = String(options.layer);
      changed();
    });
    return button;
  };
  group.append(
    el('span', undefined, 'Elevation'),
    step('−', 'Lower', -1),
    value,
    step('+', 'Raise', 1),
  );
  return group;
}

function curveToggle(options: RoadOptions, changed: () => void): HTMLButtonElement {
  const button = chipButton('Curve');
  button.id = 'road-curve';
  button.setAttribute('aria-pressed', 'false');
  button.addEventListener('click', () => {
    options.curve = !options.curve;
    button.setAttribute('aria-pressed', String(options.curve));
    changed();
  });
  return button;
}

export function createRoadOptions(changed: () => void): RoadOptionsRow {
  const options: RoadOptions = { lanes: PRESETS[0]?.lanes ?? [1, 1], layer: 0, curve: false };
  const element = el('div', 'road-options');
  element.append(
    lanePresets(options, changed),
    elevation(options, changed),
    curveToggle(options, changed),
  );
  element.hidden = true;
  return { element, options };
}
