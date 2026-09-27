import type { RoadStore } from '../road-store';
import { palette, roadStyle } from '../palette';
import type { RoadPass } from '../layers';

export interface Stroke {
  width: number;
  color: number;
}

export type StrokePass = Exclude<RoadPass, 'markings'>;

export type RoadStyle = (roads: RoadStore, road: number, pass: StrokePass) => Stroke;

const CITY_OUTLINE_FACTOR = 1.2;
const CITY_OUTLINE_PX = 1.5;
const MOTORWAY_RANK = 13;
const TRUNK_RANK = 11;
const CITY_WIDTHS = { motorway: 24, trunk: 18, other: 14 } as const;
const CITY_MIN_PX = { motorway: 3, trunk: 2.5, other: 1.6 } as const;

export function laneWidth(roads: RoadStore, road: number): number {
  return Math.max(roads.lanes(road), 1) * roadStyle.laneWidth;
}

function detailOutline(roads: RoadStore, road: number, width: number): Stroke {
  if (roads.layer(road) > 0) {
    return { width: width + roadStyle.elevatedOutlineExtra, color: palette.elevatedOutline };
  }
  return { width: width + roadStyle.outlineExtra, color: palette.roadOutline };
}

export const detailStyle: RoadStyle = (roads, road, pass) => {
  const width = laneWidth(roads, road);
  if (pass === 'outline') {
    return detailOutline(roads, road, width);
  }
  return { width, color: palette.roadFill };
};

type CityTier = keyof typeof CITY_WIDTHS;

function cityTier(rank: number): CityTier {
  if (rank >= MOTORWAY_RANK) {
    return 'motorway';
  }
  return rank >= TRUNK_RANK ? 'trunk' : 'other';
}

function cityWidth(rank: number, minScale: number): number {
  const tier = cityTier(rank);
  return Math.max(CITY_WIDTHS[tier], CITY_MIN_PX[tier] / minScale);
}

export function cityStyle(classRanks: readonly number[], minScale: number): RoadStyle {
  return (roads, road, pass) => {
    const width = cityWidth(classRanks[roads.classCode(road)] ?? 0, minScale);
    if (pass === 'outline') {
      const outline = Math.max(width * CITY_OUTLINE_FACTOR, width + CITY_OUTLINE_PX / minScale);
      return { width: outline, color: palette.roadOutline };
    }
    return { width, color: palette.roadFill };
  };
}
