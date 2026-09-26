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

export function trackPointer(canvas: HTMLCanvasElement, handlers: PointerHandlers): ScreenPosition {
  const last: ScreenPosition = { x: NaN, y: NaN };
  let down: ScreenPosition | undefined;
  let frame = 0;
  canvas.addEventListener('pointermove', (event: PointerEvent) => {
    Object.assign(last, localPoint(canvas, event));
    if (down !== undefined || frame !== 0) {
      return;
    }
    frame = requestAnimationFrame(() => {
      frame = 0;
      handlers.onHover(last.x, last.y);
    });
  });
  canvas.addEventListener('pointerdown', (event: PointerEvent) => {
    down = localPoint(canvas, event);
    handlers.onPress();
  });
  canvas.addEventListener('pointerup', (event: PointerEvent) => {
    const up = localPoint(canvas, event);
    const start = down;
    down = undefined;
    if (start !== undefined && Math.hypot(up.x - start.x, up.y - start.y) <= CLICK_SLOP_PX) {
      handlers.onClick(up.x, up.y);
    }
  });
  return last;
}
