import { roadStyle } from './palette';

export interface LaneMarkings {
  centerLine: number | undefined;
  dividers: number[];
}

function halfWidth(forward: number, backward: number): number {
  return ((forward + backward) * roadStyle.laneWidth) / 2;
}

function range(count: number): number[] {
  return Array.from({ length: Math.max(count, 0) }, (_, i) => i + 1);
}

export function laneMarkings(forward: number, backward: number): LaneMarkings {
  const half = halfWidth(forward, backward);
  const lane = roadStyle.laneWidth;
  const twoWay = forward > 0 && backward > 0;
  return {
    centerLine: twoWay ? half - forward * lane : undefined,
    dividers: [
      ...range(forward - 1).map((k) => half - k * lane),
      ...range(backward - 1).map((k) => -(half - k * lane)),
    ],
  };
}
