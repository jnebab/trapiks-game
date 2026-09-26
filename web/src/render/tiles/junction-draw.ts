import type { Graphics } from 'pixi.js';
import type { JunctionShapeArrays } from '../../sim/protocol';
import { clampLayer } from '../layers';
import { palette } from '../palette';
import { FILLET_EDGE } from '../style';
import type { PieceSet } from './piece-set';

const FILLET_POINTS = 7;

function ringRange(junctions: JunctionShapeArrays, shape: number): [number, number] {
  const start = junctions.ringStart[shape] ?? 0;
  return [start, junctions.ringStart[shape + 1] ?? start];
}

export function ringPoints(junctions: JunctionShapeArrays, shape: number): number[] {
  const [start, end] = ringRange(junctions, shape);
  const out: number[] = [];
  for (let i = start; i < end; i += 1) {
    out.push(junctions.x[i] ?? 0, junctions.y[i] ?? 0);
  }
  return out;
}

function traceFillets(g: Graphics, junctions: JunctionShapeArrays, shape: number): void {
  const [start, end] = ringRange(junctions, shape);
  for (let i = start; i < end; i += 1) {
    const x = junctions.x[i] ?? 0;
    const y = junctions.y[i] ?? 0;
    if ((i - start) % FILLET_POINTS === 0) {
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

export function drawJunctions(
  pieces: PieceSet,
  junctions: JunctionShapeArrays,
  shapes: Uint32Array,
): void {
  for (const shape of shapes) {
    const layer = clampLayer(junctions.layer[shape] ?? 0);
    const outline = pieces.get(layer, 'outline');
    traceFillets(outline, junctions, shape);
    outline.stroke({ ...edgeStyle(layer), join: 'round', cap: 'round' });
    pieces.get(layer, 'fill').poly(ringPoints(junctions, shape), true).fill(palette.roadFill);
  }
}
