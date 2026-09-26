import { Graphics, Particle, ParticleContainer, type Renderer, type Texture } from 'pixi.js';
import type { SignalPillArrays } from '../sim/protocol';
import { pillTint, pillsVisible } from './signal-pill-style';

const PX_PER_M = 20;
const LENGTH_PX = 1.4 * PX_PER_M;
const WIDTH_PX = 0.7 * PX_PER_M;
const BORDER_PX = 1.5;
const BORDER_COLOR = 0xb4b4b4;
const RESOLUTION = 4;

export interface SignalPillLayer {
  container: ParticleContainer;
  count: () => number;
  rebuild: (pills: SignalPillArrays) => void;
  setStates: (states: Uint8Array) => void;
  setScale: (scale: number) => void;
}

function createPillTexture(renderer: Renderer): Texture {
  const inset = BORDER_PX / 2;
  const target = new Graphics()
    .roundRect(inset, inset, LENGTH_PX - BORDER_PX, WIDTH_PX - BORDER_PX, WIDTH_PX / 2 - inset)
    .fill(0xffffff)
    .stroke({ width: BORDER_PX, color: BORDER_COLOR });
  const texture = renderer.generateTexture({ target, resolution: RESOLUTION, antialias: true });
  target.destroy();
  return texture;
}

function createParticle(texture: Texture, pills: SignalPillArrays, i: number): Particle {
  return new Particle({
    texture,
    x: pills.x[i] ?? 0,
    y: pills.y[i] ?? 0,
    rotation: pills.angle[i] ?? 0,
    anchorX: 0.5,
    anchorY: 0.5,
    scaleX: 1 / PX_PER_M,
    scaleY: 1 / PX_PER_M,
    tint: pillTint(2),
  });
}

export function createSignalPills(renderer: Renderer, pills: SignalPillArrays): SignalPillLayer {
  const texture = createPillTexture(renderer);
  const container = new ParticleContainer({
    texture,
    dynamicProperties: { position: false, rotation: false, color: true },
  });
  let particles: Particle[] = [];
  const rebuild = (next: SignalPillArrays): void => {
    container.removeParticles();
    particles = Array.from(next.link, (_, i) => createParticle(texture, next, i));
    particles.forEach((particle) => container.addParticle(particle));
  };
  rebuild(pills);
  const setStates = (states: Uint8Array): void => {
    particles.forEach((particle, i) => {
      particle.tint = pillTint(states[i] ?? 2);
    });
  };
  const setScale = (scale: number): void => {
    container.visible = pillsVisible(scale);
  };
  return { container, count: () => particles.length, rebuild, setStates, setScale };
}
