import type { RoadClass } from '../../generated/RoadClass';
import type { Rand } from './rng';

export type BuildingKind = 'house' | 'commercial' | 'warehouse';

const WAREHOUSE_SHARE = 0.3;

const FIXED: Partial<Record<RoadClass, BuildingKind>> = {
  Residential: 'house',
  LivingStreet: 'house',
  Primary: 'commercial',
  Secondary: 'commercial',
  Trunk: 'commercial',
};

const MIXED: readonly RoadClass[] = ['Tertiary', 'Unclassified', 'Road'];

export function buildingKind(
  roadClass: RoadClass | undefined,
  rand: Rand,
): BuildingKind | undefined {
  if (roadClass === undefined) {
    return undefined;
  }
  const fixed = FIXED[roadClass];
  if (fixed !== undefined) {
    return fixed;
  }
  if (!MIXED.includes(roadClass)) {
    return undefined;
  }
  return rand() < WAREHOUSE_SHARE ? 'warehouse' : 'house';
}

export function hasBuildings(roadClass: RoadClass | undefined): boolean {
  return roadClass !== undefined && (FIXED[roadClass] !== undefined || MIXED.includes(roadClass));
}
