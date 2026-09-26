import type { Graphics } from 'pixi.js';
import type { Marker } from '../detail-store';
import { dashes, type Point } from '../polyline';
import { SIGNAL_LINE_WIDTH, STOP_LINE, STOP_LINE_STOP_WIDTH } from '../style';
import { tracePath } from './road-markings';

const KIND_YIELD = 1;
const KIND_STOP = 2;

function markerLine(marker: Marker): Point[] {
  return [
    { x: marker.x1, y: marker.y1 },
    { x: marker.x2, y: marker.y2 },
  ];
}

function lineWidth(kind: number): number {
  if (kind === KIND_YIELD) {
    return STOP_LINE.width;
  }
  return kind === KIND_STOP ? STOP_LINE_STOP_WIDTH : SIGNAL_LINE_WIDTH;
}

export function drawMarker(g: Graphics, marker: Marker): void {
  const kind = marker.kind;
  const line = markerLine(marker);
  const dashed = kind === KIND_YIELD || kind === KIND_STOP;
  const paths = dashed ? dashes(line, STOP_LINE.dash, STOP_LINE.gap) : [line];
  for (const path of paths) {
    tracePath(g, path);
  }
  g.stroke({ width: lineWidth(kind), color: STOP_LINE.color, cap: 'butt' });
}
