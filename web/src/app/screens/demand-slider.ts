import { el } from '../../hud/dom';

const MAX_VPH = 100_000;
const STEP_VPH = 1_000;
const DEBOUNCE_MS = 300;

export interface DemandSlider {
  element: HTMLElement;
  set: (vph: number) => void;
}

function label(value: number): string {
  return `Demand ${value.toLocaleString('en-US')} vph`;
}

export function createDemandSlider(initial: number, onChange: (vph: number) => void): DemandSlider {
  const element = el('label', 'chip demand');
  const text = el('span', 'demand-label', label(initial));
  const input = el('input', 'range');
  input.id = 'demand';
  input.type = 'range';
  input.min = '0';
  input.max = String(MAX_VPH);
  input.step = String(STEP_VPH);
  input.value = String(initial);
  let timer: ReturnType<typeof setTimeout> | undefined;
  input.addEventListener('input', () => {
    text.textContent = label(Number(input.value));
  });
  input.addEventListener('change', () => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      onChange(Number(input.value));
    }, DEBOUNCE_MS);
  });
  element.append(text, input);
  return {
    element,
    set: (vph) => {
      input.value = String(vph);
      text.textContent = label(vph);
    },
  };
}
