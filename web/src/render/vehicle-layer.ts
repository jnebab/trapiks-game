import { Particle, ParticleContainer, type Texture } from 'pixi.js';
import type { VehicleFrame } from '../sim/interpolation';
import { VEHICLE_COLORS } from './palette';

const PX_TO_M = 0.1;
const FALLBACK_COLOR = 0xffffff;

export interface VehicleLayer {
  container: ParticleContainer;
  draw: (frame: VehicleFrame) => void;
}

function colorOf(style: number): number {
  return VEHICLE_COLORS[style % VEHICLE_COLORS.length] ?? FALLBACK_COLOR;
}

function place(particle: Particle, frame: VehicleFrame, i: number): void {
  particle.x = frame.x[i] ?? 0;
  particle.y = frame.y[i] ?? 0;
  particle.rotation = frame.heading[i] ?? 0;
  particle.tint = colorOf(frame.style[i] ?? 0);
  particle.alpha = 1;
}

export function createVehicleLayer(texture: Texture): VehicleLayer {
  const container = new ParticleContainer({
    texture,
    dynamicProperties: { position: true, rotation: true, color: true },
  });
  const pool: Particle[] = [];
  const grow = (count: number): void => {
    while (pool.length < count) {
      const particle = new Particle({
        texture,
        anchorX: 0.5,
        anchorY: 0.5,
        scaleX: PX_TO_M,
        scaleY: PX_TO_M,
      });
      pool.push(particle);
      container.addParticle(particle);
    }
  };
  const draw = (frame: VehicleFrame): void => {
    grow(frame.count);
    pool.forEach((particle, i) => {
      if (i < frame.count) {
        place(particle, frame, i);
      } else {
        particle.alpha = 0;
      }
    });
  };
  return { container, draw };
}
