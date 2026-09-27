import type { Graphics } from 'pixi.js';
import type { JunctionRing } from '../detail-store';
import { clampLayer } from '../layers';
import type { RoadStore } from '../road-store';
import { drawJunctions } from './junction-draw';
import { PieceSet, type TilePiece } from './piece-set';
import type { RoadStyle, Stroke, StrokePass } from './road-style';
import type { StreetData } from './street-data';
import type { TileEntry } from './tile-index';
import { drawJunctionShadows, drawRoadShadows } from './shadow-draw';
import type { DetailStore } from '../detail-store';
import { isTrimmed, traceRoad } from './trace';

export type { TilePiece } from './piece-set';

export interface TileGraphics {
  pieces: TilePiece[];
  markings: boolean;
  buildings?: Graphics;
}

type Cap = 'round' | 'butt';

interface StrokeGroup {
  layer: number;
  pass: StrokePass;
  stroke: Stroke;
  cap: Cap;
  roads: number[];
}

const PASSES: readonly StrokePass[] = ['outline', 'fill'];

function capOf(roads: RoadStore, road: number, street: StreetData | undefined): Cap {
  return street === undefined || isTrimmed(roads, road) ? 'butt' : 'round';
}

function groupRoads(
  ids: readonly number[],
  roads: RoadStore,
  style: RoadStyle,
  street: StreetData | undefined,
): StrokeGroup[] {
  const groups = new Map<string, StrokeGroup>();
  for (const road of ids) {
    const layer = clampLayer(roads.layer(road));
    const cap = capOf(roads, road, street);
    for (const pass of PASSES) {
      const stroke = style(roads, road, pass);
      const key = `${String(layer)}|${pass}|${String(stroke.width)}|${String(stroke.color)}|${cap}`;
      const group = groups.get(key) ?? { layer, pass, stroke, cap, roads: [] };
      groups.set(key, group);
      group.roads.push(road);
    }
  }
  return [...groups.values()];
}

function drawRoads(
  pieces: PieceSet,
  roads: RoadStore,
  groups: StrokeGroup[],
  detail: DetailStore | undefined,
): void {
  const join = detail === undefined ? 'miter' : 'round';
  for (const { layer, pass, stroke, cap, roads: ids } of groups) {
    const g = pieces.get(layer, pass);
    for (const road of ids) {
      traceRoad(g, roads, road, detail);
    }
    g.stroke({ width: stroke.width, color: stroke.color, join, cap });
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
  drawRoads(pieces, roads, groupRoads(live, roads, style, street), street?.detail);
  drawJunctions(pieces, rings);
  return { pieces: pieces.list(), markings: false };
}
