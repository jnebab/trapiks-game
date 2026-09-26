import { pan, screenToWorld, type Camera } from './camera';

export interface PointerSample {
  t: number;
  x: number;
  y: number;
}

export interface Velocity {
  vx: number;
  vy: number;
}

export interface FlyTarget {
  x: number;
  y: number;
  scale: number;
}

const SAMPLE_WINDOW_MS = 100;
const DECAY_PER_FRAME = 0.92;
const FRAME_MS = 16;
const STOP_SPEED = 0.01;
const STOPPED: Velocity = { vx: 0, vy: 0 };

export function releaseVelocity(samples: readonly PointerSample[]): Velocity {
  const last = samples.at(-1);
  if (last === undefined) {
    return STOPPED;
  }
  const kept = samples.filter((sample) => sample.t >= last.t - SAMPLE_WINDOW_MS);
  const first = kept[0];
  if (kept.length < 2 || first === undefined || last.t === first.t) {
    return STOPPED;
  }
  const dt = last.t - first.t;
  return { vx: (last.x - first.x) / dt, vy: (last.y - first.y) / dt };
}

export function isStopped(velocity: Velocity): boolean {
  return Math.hypot(velocity.vx, velocity.vy) < STOP_SPEED;
}

export function stepInertia(
  camera: Camera,
  velocity: Velocity,
  dtMs: number,
): { camera: Camera; velocity: Velocity } {
  if (isStopped(velocity)) {
    return { camera, velocity: STOPPED };
  }
  const moved = pan(camera, velocity.vx * dtMs, velocity.vy * dtMs);
  const decay = DECAY_PER_FRAME ** (dtMs / FRAME_MS);
  const next = { vx: velocity.vx * decay, vy: velocity.vy * decay };
  return { camera: moved, velocity: isStopped(next) ? STOPPED : next };
}

function easeInOutCubic(u: number): number {
  return u < 0.5 ? 4 * u * u * u : 1 - (-2 * u + 2) ** 3 / 2;
}

function cameraAt(target: FlyTarget, viewW: number, viewH: number): Camera {
  return {
    x: viewW / 2 - target.x * target.scale,
    y: viewH / 2 - target.y * target.scale,
    scale: target.scale,
  };
}

export function flyTo(
  from: Camera,
  target: FlyTarget,
  view: { w: number; h: number },
  durationMs: number,
): (tMs: number) => Camera {
  const [startX, startY] = screenToWorld(from, view.w / 2, view.h / 2);
  const logFrom = Math.log(from.scale);
  const logTo = Math.log(target.scale);
  const end = cameraAt(target, view.w, view.h);
  return (tMs) => {
    if (tMs >= durationMs) {
      return end;
    }
    if (tMs <= 0) {
      return from;
    }
    const e = easeInOutCubic(tMs / durationMs);
    const point = {
      x: startX + (target.x - startX) * e,
      y: startY + (target.y - startY) * e,
      scale: Math.exp(logFrom + (logTo - logFrom) * e),
    };
    return cameraAt(point, view.w, view.h);
  };
}
