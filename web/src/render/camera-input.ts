import type { Container, Ticker } from 'pixi.js';
import { MAX_SCALE, MIN_SCALE, pan, screenToWorld, zoomAt, type Camera } from './camera';
import {
  flyTo,
  releaseVelocity,
  stepInertia,
  type PointerSample,
  type Velocity,
} from './camera-motion';

export type CameraMotion =
  | { kind: 'none' }
  | { kind: 'inertia'; velocity: Velocity }
  | { kind: 'fly'; path: (tMs: number) => Camera; elapsed: number };

export interface CameraState {
  camera: Camera;
  motion?: CameraMotion;
}

const WHEEL_BASE = 1.0015;
const FLY_MS = 400;
const DOUBLE_CLICK_ZOOM = 2;
const NONE: CameraMotion = { kind: 'none' };
const limits = { min: MIN_SCALE, max: MAX_SCALE } as const;

export function applyCamera(world: Container, camera: Camera): void {
  world.position.set(camera.x, camera.y);
  world.scale.set(camera.scale);
}

function clampScale(scale: number): number {
  return Math.min(Math.max(scale, MIN_SCALE), MAX_SCALE);
}

function advance(state: CameraState, dtMs: number): void {
  const motion = state.motion ?? NONE;
  if (motion.kind === 'inertia') {
    const next = stepInertia(state.camera, motion.velocity, dtMs);
    state.camera = next.camera;
    const moving = next.velocity.vx !== 0 || next.velocity.vy !== 0;
    state.motion = moving ? { kind: 'inertia', velocity: next.velocity } : NONE;
    return;
  }
  if (motion.kind === 'fly') {
    motion.elapsed += dtMs;
    state.camera = motion.path(motion.elapsed);
    state.motion = motion.elapsed >= FLY_MS ? NONE : motion;
  }
}

export function flyCamera(
  state: CameraState,
  target: { x: number; y: number; scale: number },
  view: { w: number; h: number },
): void {
  state.motion = { kind: 'fly', path: flyTo(state.camera, target, view, FLY_MS), elapsed: 0 };
}

function wireWheel(canvas: HTMLCanvasElement, state: CameraState, world: Container): void {
  canvas.addEventListener(
    'wheel',
    (event: WheelEvent) => {
      event.preventDefault();
      state.motion = NONE;
      const factor = WHEEL_BASE ** -event.deltaY;
      state.camera = zoomAt(state.camera, [event.offsetX, event.offsetY], factor, limits);
      applyCamera(world, state.camera);
    },
    { passive: false },
  );
}

function wireDrag(canvas: HTMLCanvasElement, state: CameraState, world: Container): void {
  let samples: PointerSample[] | null = null;
  canvas.addEventListener('pointerdown', (event: PointerEvent) => {
    state.motion = NONE;
    samples = [{ t: event.timeStamp, x: event.clientX, y: event.clientY }];
    canvas.setPointerCapture(event.pointerId);
  });
  canvas.addEventListener('pointermove', (event: PointerEvent) => {
    const last = samples?.at(-1);
    if (samples === null || last === undefined) {
      return;
    }
    state.camera = pan(state.camera, event.clientX - last.x, event.clientY - last.y);
    samples.push({ t: event.timeStamp, x: event.clientX, y: event.clientY });
    applyCamera(world, state.camera);
  });
  const release = (): void => {
    const velocity = releaseVelocity(samples ?? []);
    samples = null;
    state.motion = { kind: 'inertia', velocity };
  };
  canvas.addEventListener('pointerup', release);
  canvas.addEventListener('pointercancel', release);
}

function wireDoubleClick(canvas: HTMLCanvasElement, state: CameraState): void {
  canvas.addEventListener('dblclick', (event: MouseEvent) => {
    const [x, y] = screenToWorld(state.camera, event.offsetX, event.offsetY);
    const scale = clampScale(state.camera.scale * DOUBLE_CLICK_ZOOM);
    const path = flyTo(
      state.camera,
      { x, y, scale },
      { w: canvas.clientWidth, h: canvas.clientHeight },
      FLY_MS,
    );
    state.motion = { kind: 'fly', path, elapsed: 0 };
  });
}

export function wireCameraInput(
  canvas: HTMLCanvasElement,
  state: CameraState,
  world: Container,
  ticker: Ticker,
): void {
  wireWheel(canvas, state, world);
  wireDrag(canvas, state, world);
  wireDoubleClick(canvas, state);
  ticker.add((t) => {
    if ((state.motion ?? NONE).kind === 'none') {
      return;
    }
    advance(state, t.deltaMS);
    applyCamera(world, state.camera);
  });
}
