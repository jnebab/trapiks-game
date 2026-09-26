import type { Graphics } from 'pixi.js';
import type { JunctionShapeArrays, RoadArrays } from '../../sim/protocol';
import { clampLayer } from '../layers';
import { roadStyle } from '../palette';
import { SHADOW_OFFSET } from '../shadow-style';
import { ringPoints } from './junction-draw';
import type { PieceSet } from './piece-set';
import { laneWidth } from './road-style';
import { tracePolyline } from './trace';

const SHADOW_COLOR = 0x000000;

function shadowPiece(pieces: PieceSet, layer: number): Graphics {
  const g = pieces.get(layer, 'shadow');
  g.position.set(SHADOW_OFFSET.x * layer, SHADOW_OFFSET.y * layer);
  return g;
}

export function drawRoadShadows(pieces: PieceSet, roads: RoadArrays, ids: Uint32Array): void {
  for (const road of ids) {
    const layer = clampLayer(roads.layer[road] ?? 0);
    if (layer < 1) {
      continue;
    }
    const g = shadowPiece(pieces, layer);
    tracePolyline(g, roads, road);
    g.stroke({
      width: laneWidth(roads, road) + roadStyle.elevatedOutlineExtra,
      color: SHADOW_COLOR,
      join: 'round',
      cap: 'round',
    });
  }
}

export function drawJunctionShadows(
  pieces: PieceSet,
  junctions: JunctionShapeArrays,
  shapes: Uint32Array,
): void {
  for (const shape of shapes) {
    if ((junctions.minLayer[shape] ?? 0) < 1) {
      continue;
    }
    const layer = clampLayer(junctions.layer[shape] ?? 0);
    shadowPiece(pieces, layer).poly(ringPoints(junctions, shape), true).fill(SHADOW_COLOR);
  }
}
