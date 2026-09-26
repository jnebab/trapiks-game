import type { Graphics } from 'pixi.js';
import type { JunctionRing } from '../detail-store';
import { clampLayer } from '../layers';
import { palette } from '../palette';
import { FILLET_EDGE } from '../style';
import type { PieceSet } from './piece-set';

const FILLET_POINTS = 7;

function traceFillets(g: Graphics, ring: JunctionRing): void {
  const count = ring.points.length / 2;
  for (let i = 0; i < count; i += 1) {
    const x = ring.points[i * 2] ?? 0;
    const y = ring.points[i * 2 + 1] ?? 0;
    if (i % FILLET_POINTS === 0) {
      g.moveTo(x, y);
    } else {
      g.lineTo(x, y);
    }
  }
}

function edgeStyle(layer: number): { width: number; color: number } {
  if (layer > 0) {
    return { width: FILLET_EDGE.elevated, color: palette.elevatedOutline };
  }
  return { width: FILLET_EDGE.ground, color: palette.roadOutline };
}

export function drawJunctions(pieces: PieceSet, rings: readonly JunctionRing[]): void {
  for (const ring of rings) {
    const layer = clampLayer(ring.layer);
    const outline = pieces.get(layer, 'outline');
    traceFillets(outline, ring);
    outline.stroke({ ...edgeStyle(layer), join: 'round', cap: 'round' });
    pieces.get(layer, 'fill').poly(ring.points, true).fill(palette.roadFill);
  }
}
