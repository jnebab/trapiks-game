import { VEHICLE_COLORS } from './palette';

export const VEHICLE_KINDS = 3;
const PLAIN_TINT = 0xffffff;
const COLOR_MASK = 15;
const KIND_SHIFT = 4;

export function vehicleKindOf(style: number): number {
  return Math.min(style >> KIND_SHIFT, VEHICLE_KINDS - 1);
}

export function vehicleTintOf(style: number): number {
  if (vehicleKindOf(style) !== 0) {
    return PLAIN_TINT;
  }
  return VEHICLE_COLORS[style & COLOR_MASK] ?? PLAIN_TINT;
}
