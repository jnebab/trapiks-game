import type { RoadArrays } from '../../sim/protocol';
import { clampLayer } from '../layers';
import { drawJunctions } from './junction-draw';
import { PieceSet, type TilePiece } from './piece-set';
import type { RoadStyle, Stroke, StrokePass } from './road-style';
import type { StreetData } from './street-data';
import type { TileEntry } from './tile-index';
import { drawJunctionShadows, drawRoadShadows } from './shadow-draw';
import { tracePolyline } from './trace';

export type { TilePiece } from './piece-set';

export interface TileGraphics {
  pieces: TilePiece[];
  markings: boolean;
}

interface StrokeGroup {
  layer: number;
  pass: StrokePass;
  stroke: Stroke;
  roads: number[];
}

const PASSES: readonly StrokePass[] = ['outline', 'fill'];

function groupRoads(entry: TileEntry, roads: RoadArrays, style: RoadStyle): StrokeGroup[] {
  const groups = new Map<string, StrokeGroup>();
  for (const road of entry.roads) {
    const layer = clampLayer(roads.layer[road] ?? 0);
    for (const pass of PASSES) {
      const stroke = style(roads, road, pass);
      const key = `${String(layer)}|${pass}|${String(stroke.width)}|${String(stroke.color)}`;
      const group = groups.get(key) ?? { layer, pass, stroke, roads: [] };
      groups.set(key, group);
      group.roads.push(road);
    }
  }
  return [...groups.values()];
}

function drawRoads(pieces: PieceSet, roads: RoadArrays, groups: StrokeGroup[]): void {
  for (const { layer, pass, stroke, roads: ids } of groups) {
    const g = pieces.get(layer, pass);
    for (const road of ids) {
      tracePolyline(g, roads, road);
    }
    g.stroke({ width: stroke.width, color: stroke.color, join: 'round', cap: 'round' });
  }
}

export function buildTile(
  entry: TileEntry,
  roads: RoadArrays,
  style: RoadStyle,
  street?: StreetData,
): TileGraphics {
  const pieces = new PieceSet();
  if (street !== undefined) {
    drawRoadShadows(pieces, roads, entry.roads);
    drawJunctionShadows(pieces, street.junctions, entry.junctions);
  }
  drawRoads(pieces, roads, groupRoads(entry, roads, style));
  if (street !== undefined) {
    drawJunctions(pieces, street.junctions, entry.junctions);
  }
  return { pieces: pieces.list(), markings: false };
}
