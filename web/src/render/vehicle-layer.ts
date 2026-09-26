import { Container, Particle, ParticleContainer, type Texture } from 'pixi.js';
import type { VehicleFrame } from '../sim/interpolation';
import type { VehicleAtlas } from './vehicle-atlas';
import { VEHICLE_KINDS, vehicleKindOf, vehicleTintOf } from './vehicle-style';

const PX_TO_M = 0.1;
const CAR_KIND = 0;

export interface VehicleLayer {
  container: Container;
  draw: (frame: VehicleFrame) => void;
}

interface Pool {
  container: ParticleContainer;
  texture: Texture;
  particles: Particle[];
  used: number;
}

function createPool(texture: Texture, tinted: boolean): Pool {
  const container = new ParticleContainer({
    texture,
    dynamicProperties: { position: true, rotation: true, color: tinted },
  });
  return { container, texture, particles: [], used: 0 };
}

function nextParticle(pool: Pool): Particle {
  const existing = pool.particles[pool.used];
  pool.used += 1;
  if (existing !== undefined) {
    return existing;
  }
  const particle = new Particle({
    texture: pool.texture,
    anchorX: 1,
    anchorY: 0.5,
    scaleX: PX_TO_M,
    scaleY: PX_TO_M,
  });
  pool.particles.push(particle);
  pool.container.addParticle(particle);
  return particle;
}

function place(pool: Pool, frame: VehicleFrame, i: number, style: number): void {
  const particle = nextParticle(pool);
  particle.x = frame.x[i] ?? 0;
  particle.y = frame.y[i] ?? 0;
  particle.rotation = frame.heading[i] ?? 0;
  particle.tint = vehicleTintOf(style);
  particle.alpha = 1;
}

function hideUnused(pool: Pool): void {
  for (let i = pool.used; i < pool.particles.length; i += 1) {
    const particle = pool.particles[i];
    if (particle !== undefined) {
      particle.alpha = 0;
    }
  }
}

export function createVehicleLayer(atlas: VehicleAtlas): VehicleLayer {
  const container = new Container();
  const pools: Pool[] = atlas
    .slice(0, VEHICLE_KINDS)
    .map((texture, kind) => createPool(texture, kind === CAR_KIND));
  pools.forEach((pool) => container.addChild(pool.container));
  const draw = (frame: VehicleFrame): void => {
    pools.forEach((pool) => {
      pool.used = 0;
    });
    for (let i = 0; i < frame.count; i += 1) {
      const style = frame.style[i] ?? 0;
      const pool = pools[vehicleKindOf(style)];
      if (pool !== undefined) {
        place(pool, frame, i, style);
      }
    }
    pools.forEach(hideUnused);
  };
  return { container, draw };
}
