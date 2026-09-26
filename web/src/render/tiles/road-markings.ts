import type { Graphics } from 'pixi.js';
import type { DetailStore } from '../detail-store';
import type { RoadStore } from '../road-store';
import { laneMarkings } from '../markings';
import {
  arrowsAlong,
  cumulative,
  dashes,
  offset,
  slice,
  type Arrow,
  type Point,
} from '../polyline';
import { ARROW, ARROW_SPACING, ARROW_START, CENTER_LINE, LANE_DIVIDER } from '../style';

function trimmed(roads: RoadStore, detail: DetailStore, road: number): Point[] {
  const points = roads.pointsOf(road);
  const lengths = cumulative(points);
  const total = lengths[lengths.length - 1] ?? 0;
  const from = detail.setback(road, 0);
  const to = total - detail.setback(road, 1);
  return slice(points, from, to);
}

export function tracePath(g: Graphics, points: readonly Point[]): void {
  points.forEach((p, i) => {
    if (i === 0) {
      g.moveTo(p.x, p.y);
    } else {
      g.lineTo(p.x, p.y);
    }
  });
}

function drawDividers(g: Graphics, span: readonly Point[], offsets: readonly number[]): void {
  if (offsets.length === 0) {
    return;
  }
  for (const d of offsets) {
    dashes(offset(span, d), LANE_DIVIDER.dash, LANE_DIVIDER.gap).forEach((dash) => {
      tracePath(g, dash);
    });
  }
  g.stroke({ width: LANE_DIVIDER.width, color: LANE_DIVIDER.color, cap: 'butt' });
}

function drawCenterLine(g: Graphics, span: readonly Point[], d: number | undefined): void {
  if (d === undefined) {
    return;
  }
  tracePath(g, offset(span, d));
  g.stroke({ width: CENTER_LINE.width, color: CENTER_LINE.color, cap: 'butt', join: 'round' });
}

function oneWayArrows(span: readonly Point[], forward: number): Arrow[] {
  const directed = forward > 0 ? [...span] : [...span].reverse();
  const arrows = arrowsAlong(directed, ARROW_SPACING, ARROW_START);
  if (arrows.length > 0) {
    return arrows;
  }
  const lengths = cumulative(directed);
  return arrowsAlong(directed, ARROW_SPACING, (lengths[lengths.length - 1] ?? 0) / 2);
}

function arrowTriangle(a: Arrow): number[] {
  const cos = Math.cos(a.angle);
  const sin = Math.sin(a.angle);
  const half = ARROW.length / 2;
  const side = ARROW.width / 2;
  return [
    a.x + cos * half,
    a.y + sin * half,
    a.x - cos * half - sin * side,
    a.y - sin * half + cos * side,
    a.x - cos * half + sin * side,
    a.y - sin * half - cos * side,
  ];
}

function drawArrows(g: Graphics, span: readonly Point[], forward: number): void {
  for (const arrow of oneWayArrows(span, forward)) {
    g.poly(arrowTriangle(arrow), true).fill(ARROW.color);
  }
}

export function drawRoadMarkings(
  g: Graphics,
  roads: RoadStore,
  detail: DetailStore,
  road: number,
): void {
  const span = trimmed(roads, detail, road);
  if (span.length < 2) {
    return;
  }
  const forward = roads.lanesForward(road);
  const backward = roads.lanesBackward(road);
  const lanes = laneMarkings(forward, backward);
  drawDividers(g, span, lanes.dividers);
  drawCenterLine(g, span, lanes.centerLine);
  if (lanes.centerLine === undefined && forward + backward > 0) {
    drawArrows(g, span, forward);
  }
}
