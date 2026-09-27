import type { Graphics } from 'pixi.js';
import { FILLET_POINTS, type JunctionRing } from '../detail-store';
import { clampLayer } from '../layers';
import { palette } from '../palette';
import { FILLET_EDGE } from '../style';
import type { PieceSet } from './piece-set';

function traceFillet(g: Graphics, ring: JunctionRing, fillet: number): void {
  const first = fillet * FILLET_POINTS;
  for (let i = 0; i < FILLET_POINTS; i += 1) {
    const x = ring.points[(first + i) * 2] ?? 0;
    const y = ring.points[(first + i) * 2 + 1] ?? 0;
    if (i === 0) {
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

function isMixed(ring: JunctionRing): boolean {
  return clampLayer(ring.layer) !== clampLayer(ring.minLayer);
}

function filletLayer(ring: JunctionRing, fillet: number, base: number): number {
  if (!isMixed(ring)) {
    return base;
  }
  const layer = clampLayer(ring.filletLayers[fillet] ?? base);
  return layer > 0 ? layer : base;
}

function strokeFillets(pieces: PieceSet, ring: JunctionRing, base: number): void {
  const count = ring.points.length / 2 / FILLET_POINTS;
  const layers = new Set<number>();
  for (let fillet = 0; fillet < count; fillet += 1) {
    const layer = filletLayer(ring, fillet, base);
    traceFillet(pieces.get(layer, 'outline'), ring, fillet);
    layers.add(layer);
  }
  for (const layer of layers) {
    pieces.get(layer, 'outline').stroke({ ...edgeStyle(layer), join: 'round', cap: 'round' });
  }
}

export function drawJunctions(pieces: PieceSet, rings: readonly JunctionRing[]): void {
  for (const ring of rings) {
    const base = clampLayer(isMixed(ring) ? ring.minLayer : ring.layer);
    strokeFillets(pieces, ring, base);
    pieces.get(base, 'fill').poly(ring.points, true).fill(palette.roadFill);
  }
}
