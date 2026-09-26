import type { RoadArrays } from '../../sim/protocol';
import { palette, roadStyle } from '../palette';
import type { RoadPass } from '../layers';

export interface Stroke {
  width: number;
  color: number;
}

export type StrokePass = Exclude<RoadPass, 'markings'>;

export type RoadStyle = (roads: RoadArrays, road: number, pass: StrokePass) => Stroke;

const CITY_OUTLINE_FACTOR = 1.2;
const MOTORWAY_RANK = 13;
const TRUNK_RANK = 11;
const CITY_WIDTHS = { motorway: 24, trunk: 18, other: 14 } as const;

function laneWidth(roads: RoadArrays, road: number): number {
  const lanes = (roads.lanesForward[road] ?? 0) + (roads.lanesBackward[road] ?? 0);
  return Math.max(lanes, 1) * roadStyle.laneWidth;
}

function detailOutline(roads: RoadArrays, road: number, width: number): Stroke {
  if ((roads.layer[road] ?? 0) > 0) {
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

function cityWidth(rank: number): number {
  if (rank >= MOTORWAY_RANK) {
    return CITY_WIDTHS.motorway;
  }
  return rank >= TRUNK_RANK ? CITY_WIDTHS.trunk : CITY_WIDTHS.other;
}

export function cityStyle(classRanks: readonly number[]): RoadStyle {
  return (roads, road, pass) => {
    const width = cityWidth(classRanks[roads.classCode[road] ?? 0] ?? 0);
    if (pass === 'outline') {
      return { width: width * CITY_OUTLINE_FACTOR, color: palette.roadOutline };
    }
    return { width, color: palette.roadFill };
  };
}
