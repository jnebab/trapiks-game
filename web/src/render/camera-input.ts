import type { Container } from 'pixi.js';
import { MAX_SCALE, MIN_SCALE, pan, zoomAt, type Camera } from './camera';

export interface CameraState {
  camera: Camera;
}

const WHEEL_BASE = 1.0015;
const limits = { min: MIN_SCALE, max: MAX_SCALE } as const;

export function applyCamera(world: Container, camera: Camera): void {
  world.position.set(camera.x, camera.y);
  world.scale.set(camera.scale);
}

function wireWheel(canvas: HTMLCanvasElement, state: CameraState, world: Container): void {
  canvas.addEventListener(
    'wheel',
    (event: WheelEvent) => {
      event.preventDefault();
      const factor = WHEEL_BASE ** -event.deltaY;
      state.camera = zoomAt(state.camera, [event.offsetX, event.offsetY], factor, limits);
      applyCamera(world, state.camera);
    },
    { passive: false },
  );
}

function wireDrag(canvas: HTMLCanvasElement, state: CameraState, world: Container): void {
  let last: { x: number; y: number } | null = null;
  canvas.addEventListener('pointerdown', (event: PointerEvent) => {
    last = { x: event.clientX, y: event.clientY };
    canvas.setPointerCapture(event.pointerId);
  });
  canvas.addEventListener('pointermove', (event: PointerEvent) => {
    if (last === null) {
      return;
    }
    state.camera = pan(state.camera, event.clientX - last.x, event.clientY - last.y);
    last = { x: event.clientX, y: event.clientY };
    applyCamera(world, state.camera);
  });
  const release = (): void => {
    last = null;
  };
  canvas.addEventListener('pointerup', release);
  canvas.addEventListener('pointercancel', release);
}

export function wireCameraInput(
  canvas: HTMLCanvasElement,
  state: CameraState,
  world: Container,
): void {
  wireWheel(canvas, state, world);
  wireDrag(canvas, state, world);
}
