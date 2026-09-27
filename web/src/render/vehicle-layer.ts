import { Particle, ParticleContainer, type Container, type Texture } from 'pixi.js';
import type { VehicleFrame } from '../sim/interpolation';
import { layerIndex, MIN_LAYER } from './layer-range';
import type { VehicleAtlas } from './vehicle-atlas';
import { VEHICLE_KINDS, vehicleKindOf, vehicleTintOf } from './vehicle-style';

const PX_TO_M = 0.1;
const CAR_KIND = 0;

export type LayerContainer = (layer: number) => Container;

export interface VehicleLayer {
  draw: (frame: VehicleFrame) => void;
  layerCounts: () => string;
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

function createKindPools(atlas: VehicleAtlas, parent: Container): Pool[] {
  const pools = atlas
    .slice(0, VEHICLE_KINDS)
    .map((texture, kind) => createPool(texture, kind === CAR_KIND));
  pools.forEach((pool) => parent.addChild(pool.container));
  return pools;
}

function usedCount(pools: Pool[]): number {
  return pools.reduce((sum, pool) => sum + pool.used, 0);
}

function summarize(byLayer: Map<number, Pool[]>): string {
  return [...byLayer.keys()]
    .sort((a, b) => a - b)
    .map((index) => `${String(index + MIN_LAYER)}:${String(usedCount(byLayer.get(index) ?? []))}`)
    .join(',');
}

export function createVehicleLayer(
  atlas: VehicleAtlas,
  containerFor: LayerContainer,
): VehicleLayer {
  const byLayer = new Map<number, Pool[]>();
  const poolsFor = (layer: number): Pool[] => {
    const index = layerIndex(layer);
    const existing = byLayer.get(index);
    if (existing !== undefined) {
      return existing;
    }
    const created = createKindPools(atlas, containerFor(layer));
    byLayer.set(index, created);
    return created;
  };
  const forEachPool = (visit: (pool: Pool) => void): void => {
    byLayer.forEach((pools) => {
      pools.forEach(visit);
    });
  };
  const draw = (frame: VehicleFrame): void => {
    forEachPool((pool) => {
      pool.used = 0;
    });
    for (let i = 0; i < frame.count; i += 1) {
      const style = frame.style[i] ?? 0;
      const pool = poolsFor(frame.layer[i] ?? 0)[vehicleKindOf(style)];
      if (pool !== undefined) {
        place(pool, frame, i, style);
      }
    }
    forEachPool(hideUnused);
  };
  return { draw, layerCounts: () => summarize(byLayer) };
}
