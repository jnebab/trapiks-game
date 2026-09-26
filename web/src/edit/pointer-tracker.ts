const CLICK_SLOP_PX = 4;

export interface PointerHandlers {
  onHover: (sx: number, sy: number) => void;
  onClick: (sx: number, sy: number) => void;
  onPress: () => void;
}

export interface ScreenPosition {
  x: number;
  y: number;
}

function localPoint(canvas: HTMLCanvasElement, event: PointerEvent): ScreenPosition {
  const rect = canvas.getBoundingClientRect();
  return { x: event.clientX - rect.left, y: event.clientY - rect.top };
}

interface PointerState {
  last: ScreenPosition;
  down: ScreenPosition | undefined;
  frame: number;
}

function onMove(canvas: HTMLCanvasElement, handlers: PointerHandlers, state: PointerState) {
  return (event: PointerEvent): void => {
    Object.assign(state.last, localPoint(canvas, event));
    if (state.down !== undefined || state.frame !== 0) {
      return;
    }
    state.frame = requestAnimationFrame(() => {
      state.frame = 0;
      handlers.onHover(state.last.x, state.last.y);
    });
  };
}

function onUp(canvas: HTMLCanvasElement, handlers: PointerHandlers, state: PointerState) {
  return (event: PointerEvent): void => {
    const up = localPoint(canvas, event);
    const start = state.down;
    state.down = undefined;
    if (start !== undefined && Math.hypot(up.x - start.x, up.y - start.y) <= CLICK_SLOP_PX) {
      handlers.onClick(up.x, up.y);
    }
  };
}

export function trackPointer(
  canvas: HTMLCanvasElement,
  handlers: PointerHandlers,
  signal: AbortSignal,
): ScreenPosition {
  const state: PointerState = { last: { x: NaN, y: NaN }, down: undefined, frame: 0 };
  const onDown = (event: PointerEvent): void => {
    state.down = localPoint(canvas, event);
    handlers.onPress();
  };
  canvas.addEventListener('pointermove', onMove(canvas, handlers, state), { signal });
  canvas.addEventListener('pointerdown', onDown, { signal });
  canvas.addEventListener('pointerup', onUp(canvas, handlers, state), { signal });
  return state.last;
}
