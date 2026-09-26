import { Graphics } from 'pixi.js';
import type { RoadArrays } from '../../sim/protocol';
import { clampLayer, type RoadPass } from '../layers';
import type { RoadStyle, Stroke } from './road-style';
import type { TileEntry } from './tile-index';

export interface TilePiece {
  layer: number;
  pass: RoadPass;
  graphics: Graphics;
}

export interface TileGraphics {
  pieces: TilePiece[];
}

interface StrokeGroup {
  stroke: Stroke;
  roads: number[];
}

type PieceGroups = Map<string, Map<string, StrokeGroup>>;

const PASSES: readonly RoadPass[] = ['outline', 'fill'];

function pieceKey(layer: number, pass: RoadPass): string {
  return `${String(layer)}|${pass}`;
}

function addToGroup(groups: PieceGroups, piece: string, stroke: Stroke, road: number): void {
  const strokes = groups.get(piece) ?? new Map<string, StrokeGroup>();
  groups.set(piece, strokes);
  const key = `${String(stroke.width)}|${String(stroke.color)}`;
  const group = strokes.get(key) ?? { stroke, roads: [] };
  strokes.set(key, group);
  group.roads.push(road);
}

function tracePolyline(g: Graphics, roads: RoadArrays, road: number): void {
  const start = roads.pointStart[road] ?? 0;
  const end = roads.pointStart[road + 1] ?? start;
  g.moveTo(roads.x[start] ?? 0, roads.y[start] ?? 0);
  for (let i = start + 1; i < end; i += 1) {
    g.lineTo(roads.x[i] ?? 0, roads.y[i] ?? 0);
  }
}

function drawPiece(roads: RoadArrays, strokes: Map<string, StrokeGroup>): Graphics {
  const g = new Graphics();
  for (const { stroke, roads: ids } of strokes.values()) {
    for (const road of ids) {
      tracePolyline(g, roads, road);
    }
    g.stroke({ width: stroke.width, color: stroke.color, join: 'round', cap: 'round' });
  }
  return g;
}

function groupRoads(entry: TileEntry, roads: RoadArrays, style: RoadStyle): PieceGroups {
  const groups: PieceGroups = new Map();
  for (const road of entry.roads) {
    const layer = clampLayer(roads.layer[road] ?? 0);
    for (const pass of PASSES) {
      addToGroup(groups, pieceKey(layer, pass), style(roads, road, pass), road);
    }
  }
  return groups;
}

export function buildTile(entry: TileEntry, roads: RoadArrays, style: RoadStyle): TileGraphics {
  const pieces: TilePiece[] = [];
  for (const [key, strokes] of groupRoads(entry, roads, style)) {
    const [layer, pass] = key.split('|');
    pieces.push({
      layer: Number(layer),
      pass: pass === 'outline' ? 'outline' : 'fill',
      graphics: drawPiece(roads, strokes),
    });
  }
  return { pieces };
}
