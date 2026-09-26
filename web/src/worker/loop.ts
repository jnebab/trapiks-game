export interface WallTicks {
  stop: () => void;
}

export function startWallTicks(onTick: () => void, periodMs: number): WallTicks {
  let next = Date.now() + periodMs;
  let handle: ReturnType<typeof setTimeout> | undefined;
  const tick = (): void => {
    onTick();
    next += periodMs;
    const now = Date.now();
    if (now - next > periodMs) {
      next = now;
    }
    handle = setTimeout(tick, Math.max(0, next - now));
  };
  handle = setTimeout(tick, periodMs);
  return {
    stop: () => {
      clearTimeout(handle);
    },
  };
}
