import type { JunctionRing } from '../detail-store';
import { clampLayer } from '../layers';
import type { RoadStore } from '../road-store';
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

function groupRoads(ids: readonly number[], roads: RoadStore, style: RoadStyle): StrokeGroup[] {
  const groups = new Map<string, StrokeGroup>();
  for (const road of ids) {
    const layer = clampLayer(roads.layer(road));
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

function drawRoads(pieces: PieceSet, roads: RoadStore, groups: StrokeGroup[]): void {
  for (const { layer, pass, stroke, roads: ids } of groups) {
    const g = pieces.get(layer, pass);
    for (const road of ids) {
      tracePolyline(g, roads, road);
    }
    g.stroke({ width: stroke.width, color: stroke.color, join: 'round', cap: 'round' });
  }
}

function ringsOf(entry: TileEntry, street: StreetData): JunctionRing[] {
  return entry.junctions
    .map((node) => street.detail.junctions.get(node))
    .filter((ring) => ring !== undefined);
}

export function buildTile(
  entry: TileEntry,
  roads: RoadStore,
  style: RoadStyle,
  street?: StreetData,
): TileGraphics {
  const pieces = new PieceSet();
  const live = entry.roads.filter((road) => !roads.isDeleted(road));
  const rings = street === undefined ? [] : ringsOf(entry, street);
  if (street !== undefined) {
    drawRoadShadows(pieces, roads, live);
    drawJunctionShadows(pieces, rings);
  }
  drawRoads(pieces, roads, groupRoads(live, roads, style));
  drawJunctions(pieces, rings);
  return { pieces: pieces.list(), markings: false };
}
