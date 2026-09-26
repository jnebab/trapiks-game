import type { Graphics } from 'pixi.js';
import type { JunctionRing } from '../detail-store';
import { clampLayer } from '../layers';
import { roadStyle } from '../palette';
import type { RoadStore } from '../road-store';
import { SHADOW_OFFSET } from '../shadow-style';
import type { PieceSet } from './piece-set';
import { laneWidth } from './road-style';
import { tracePolyline } from './trace';

const SHADOW_COLOR = 0x000000;

function shadowPiece(pieces: PieceSet, layer: number): Graphics {
  const g = pieces.get(layer, 'shadow');
  g.position.set(SHADOW_OFFSET.x * layer, SHADOW_OFFSET.y * layer);
  return g;
}

export function drawRoadShadows(pieces: PieceSet, roads: RoadStore, ids: readonly number[]): void {
  for (const road of ids) {
    const layer = clampLayer(roads.layer(road));
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

export function drawJunctionShadows(pieces: PieceSet, rings: readonly JunctionRing[]): void {
  for (const ring of rings) {
    if (ring.minLayer < 1) {
      continue;
    }
    const layer = clampLayer(ring.layer);
    shadowPiece(pieces, layer).poly(ring.points, true).fill(SHADOW_COLOR);
  }
}
