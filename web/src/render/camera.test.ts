import { describe, expect, it } from 'vitest';
import { MAX_SCALE, MIN_SCALE, fitBounds, screenToWorld, worldToScreen, zoomAt } from './camera';

const limits = { min: MIN_SCALE, max: MAX_SCALE };

describe('camera', () => {
  it('fitBounds centres the bounds', () => {
    const camera = fitBounds([100, 200, 300, 600], 800, 600, 20);
    const [sx, sy] = worldToScreen(camera, 200, 400);
    expect(sx).toBeCloseTo(400, 9);
    expect(sy).toBeCloseTo(300, 9);
    expect(camera.scale).toBeCloseTo(560 / 400, 9);
  });

  it('zoomAt keeps the cursor world point fixed', () => {
    const camera = { x: 13, y: -7, scale: 0.8 };
    const before = screenToWorld(camera, 250, 120);
    const zoomed = zoomAt(camera, [250, 120], 1.7, limits);
    const after = screenToWorld(zoomed, 250, 120);
    expect(Math.abs(after[0] - before[0])).toBeLessThan(1e-9);
    expect(Math.abs(after[1] - before[1])).toBeLessThan(1e-9);
  });

  it('screenToWorld inverts worldToScreen', () => {
    const camera = { x: 40, y: 90, scale: 2.5 };
    const [sx, sy] = worldToScreen(camera, -12.5, 33);
    const [wx, wy] = screenToWorld(camera, sx, sy);
    expect(wx).toBeCloseTo(-12.5, 9);
    expect(wy).toBeCloseTo(33, 9);
  });

  it('clamps the scale', () => {
    const camera = { x: 0, y: 0, scale: 1 };
    expect(zoomAt(camera, [0, 0], 1000, limits).scale).toBe(MAX_SCALE);
    expect(zoomAt(camera, [0, 0], 0.0001, limits).scale).toBe(MIN_SCALE);
    expect(fitBounds([0, 0, 1e7, 1e7], 800, 600, 10).scale).toBe(MIN_SCALE);
    expect(fitBounds([0, 0, 1, 1], 800, 600, 10).scale).toBe(MAX_SCALE);
  });
});
