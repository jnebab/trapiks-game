import { describe, expect, it } from 'vitest';
import { screenToWorld } from './camera';
import { flyTo, releaseVelocity, stepInertia } from './camera-motion';

describe('releaseVelocity', () => {
  it('uses only the last 100 ms of samples', () => {
    const samples = [
      { t: 0, x: 0, y: 0 },
      { t: 150, x: 100, y: 0 },
      { t: 200, x: 110, y: 20 },
      { t: 250, x: 120, y: 40 },
    ];
    expect(releaseVelocity(samples)).toEqual({ vx: 0.2, vy: 0.4 });
  });

  it('returns zero without enough motion', () => {
    expect(releaseVelocity([{ t: 5, x: 1, y: 1 }])).toEqual({ vx: 0, vy: 0 });
    expect(releaseVelocity([])).toEqual({ vx: 0, vy: 0 });
    const same = [
      { t: 5, x: 1, y: 1 },
      { t: 5, x: 9, y: 9 },
    ];
    expect(releaseVelocity(same)).toEqual({ vx: 0, vy: 0 });
  });
});

describe('stepInertia', () => {
  it('pans and decays to a stop', () => {
    let state = { camera: { x: 0, y: 0, scale: 1 }, velocity: { vx: 2, vy: 0 } };
    state = stepInertia(state.camera, state.velocity, 16);
    expect(state.camera.x).toBeCloseTo(32);
    expect(state.velocity.vx).toBeCloseTo(1.84);
    let frames = 0;
    while (state.velocity.vx !== 0 && frames < 1000) {
      state = stepInertia(state.camera, state.velocity, 16);
      frames += 1;
    }
    expect(state.velocity).toEqual({ vx: 0, vy: 0 });
    expect(frames).toBeLessThan(100);
  });
});

describe('flyTo', () => {
  const from = { x: 10, y: -20, scale: 0.5 };
  const target = { x: 300, y: 200, scale: 8 };
  const fly = flyTo(from, target, { w: 800, h: 600 }, 400);

  it('starts and ends exactly at the endpoints', () => {
    expect(fly(0)).toEqual(from);
    const end = fly(400);
    expect(end.scale).toBe(8);
    expect(screenToWorld(end, 400, 300)).toEqual([300, 200]);
    expect(fly(900)).toEqual(end);
  });

  it('interpolates scale in log space', () => {
    expect(fly(200).scale).toBeCloseTo(Math.sqrt(0.5 * 8));
  });
});
