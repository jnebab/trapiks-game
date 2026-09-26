import type { Speed } from '../sim/protocol';
import { el } from './dom';
import { isEditableTarget } from './keys';

const BUTTONS: readonly { speed: Speed; label: string }[] = [
  { speed: 0, label: '⏸' },
  { speed: 1, label: '1×' },
  { speed: 2, label: '2×' },
  { speed: 4, label: '4×' },
  { speed: 8, label: '8×' },
  { speed: 'max', label: 'Max' },
];

const DIGIT_SPEEDS: Readonly<Record<string, Speed>> = {
  Digit1: 1,
  Digit2: 2,
  Digit3: 4,
  Digit4: 8,
  Digit5: 'max',
};

export interface SpeedControl {
  element: HTMLElement;
}

interface SpeedState {
  speed: Speed;
  resume: Speed;
}

function keySpeed(event: KeyboardEvent, state: SpeedState): Speed | undefined {
  if (event.code === 'Space') {
    return state.speed === 0 ? state.resume : 0;
  }
  return DIGIT_SPEEDS[event.code];
}

export function createSpeedControl(
  debug: HTMLElement,
  onChange: (speed: Speed) => void,
): SpeedControl {
  const element = el('div', 'speed-control');
  const state: SpeedState = { speed: 1, resume: 1 };
  const buttons = BUTTONS.map(({ speed, label }) => {
    const button = el('button', 'chip', label);
    button.type = 'button';
    button.addEventListener('click', () => {
      select(speed);
    });
    element.appendChild(button);
    return { speed, button };
  });
  function select(speed: Speed): void {
    state.speed = speed;
    if (speed !== 0) {
      state.resume = speed;
    }
    buttons.forEach((entry) => entry.button.classList.toggle('active', entry.speed === speed));
    debug.dataset.speed = String(speed);
    onChange(speed);
  }
  window.addEventListener('keydown', (event) => {
    if (isEditableTarget(event)) {
      return;
    }
    const speed = keySpeed(event, state);
    if (speed === undefined) {
      return;
    }
    event.preventDefault();
    select(speed);
  });
  select(1);
  return { element };
}
