import type { Graphics } from 'pixi.js';
import type { ApproachMarkerArrays } from '../../sim/protocol';
import { dashes, type Point } from '../polyline';
import { SIGNAL_LINE_WIDTH, STOP_LINE, STOP_LINE_STOP_WIDTH } from '../style';
import { tracePath } from './road-markings';

const KIND_YIELD = 1;
const KIND_STOP = 2;

function markerLine(markers: ApproachMarkerArrays, marker: number): Point[] {
  return [
    { x: markers.x1[marker] ?? 0, y: markers.y1[marker] ?? 0 },
    { x: markers.x2[marker] ?? 0, y: markers.y2[marker] ?? 0 },
  ];
}

function lineWidth(kind: number): number {
  if (kind === KIND_YIELD) {
    return STOP_LINE.width;
  }
  return kind === KIND_STOP ? STOP_LINE_STOP_WIDTH : SIGNAL_LINE_WIDTH;
}

export function drawMarker(g: Graphics, markers: ApproachMarkerArrays, marker: number): void {
  const kind = markers.kind[marker] ?? 0;
  const line = markerLine(markers, marker);
  const dashed = kind === KIND_YIELD || kind === KIND_STOP;
  const paths = dashed ? dashes(line, STOP_LINE.dash, STOP_LINE.gap) : [line];
  for (const path of paths) {
    tracePath(g, path);
  }
  g.stroke({ width: lineWidth(kind), color: STOP_LINE.color, cap: 'butt' });
}
