import type { Graphics } from 'pixi.js';
import { clampLayer } from '../layer-range';
import { cumulative, pointAt, type Point } from '../polyline';
import type { RoadStore } from '../road-store';
import { badgeDistances, endLevel, rampBadges, type RampBadge } from './badge-placement';
import { GLYPH_WIDTH, glyphOf } from './badge-glyphs';
import type { PieceSet } from './piece-set';
import { tracePath } from './road-markings';

export const MAX_BADGES_PER_TILE = 96;

const HEIGHT = 5.4;
const TEXT_HEIGHT = 3;
const CHAR_GAP = 0.7;
const PADDING = 1.4;
const TEXT_STROKE = 0.62;
const BORDER = 0.28;
const TEXT_COLOR = 0xffffff;
const LAYER_COLORS: readonly number[] = [
  0x5c6470, 0x2f6fb0, 0x7a4fa0, 0xa0508a, 0xb0603a, 0x8a6a20,
];
const RAMP_COLOR = 0x2b8a5a;

export interface BadgeBudget {
  remaining: number;
}

interface Badge {
  at: Point;
  text: string;
  color: number;
}

function textWidth(text: string): number {
  const chars = text.length;
  return chars * GLYPH_WIDTH * TEXT_HEIGHT + Math.max(chars - 1, 0) * CHAR_GAP;
}

function drawText(g: Graphics, badge: Badge): void {
  const width = textWidth(badge.text);
  let left = badge.at.x - width / 2;
  const top = badge.at.y - TEXT_HEIGHT / 2;
  for (const char of badge.text) {
    for (const stroke of glyphOf(char)) {
      tracePath(
        g,
        stroke.map((p) => ({ x: left + p.x * TEXT_HEIGHT, y: top + p.y * TEXT_HEIGHT })),
      );
    }
    left += GLYPH_WIDTH * TEXT_HEIGHT + CHAR_GAP;
  }
  g.stroke({ width: TEXT_STROKE, color: TEXT_COLOR, cap: 'round', join: 'round' });
}

function drawBadge(g: Graphics, badge: Badge): void {
  const width = Math.max(textWidth(badge.text) + 2 * PADDING, HEIGHT);
  const { x, y } = badge.at;
  g.roundRect(x - width / 2, y - HEIGHT / 2, width, HEIGHT, HEIGHT / 2)
    .fill(badge.color)
    .stroke({ width: BORDER, color: TEXT_COLOR });
  drawText(g, badge);
}

function levelColor(layer: number): number {
  return LAYER_COLORS[Math.min(layer, LAYER_COLORS.length - 1)] ?? RAMP_COLOR;
}

function levelBadges(points: readonly Point[], lengths: readonly number[], layer: number): Badge[] {
  if (layer < 1) {
    return [];
  }
  const total = lengths[lengths.length - 1] ?? 0;
  const text = `L${String(layer)}`;
  return badgeDistances(total).map((s) => ({
    at: pointAt(points, lengths, s),
    text,
    color: levelColor(layer),
  }));
}

function neighbourLayers(roads: RoadStore, road: number, node: number): number[] {
  return roads
    .roadsAt(node)
    .filter((other) => other !== road)
    .map((other) => clampLayer(roads.layer(other)));
}

function ramps(roads: RoadStore, road: number, length: number): RampBadge[] {
  const own = clampLayer(roads.layer(road));
  return rampBadges({
    length,
    fromLevel: endLevel(own, neighbourLayers(roads, road, roads.from(road))),
    toLevel: endLevel(own, neighbourLayers(roads, road, roads.to(road))),
    forward: roads.lanesForward(road) > 0,
    backward: roads.lanesBackward(road) > 0,
  });
}

function rampBadgeAt(points: readonly Point[], lengths: readonly number[], ramp: RampBadge): Badge {
  return {
    at: pointAt(points, lengths, ramp.distance),
    text: ramp.climb === 'up' ? '↑' : '↓',
    color: RAMP_COLOR,
  };
}

function roadBadges(roads: RoadStore, road: number): Badge[] {
  const points = roads.pointsOf(road);
  if (points.length < 2) {
    return [];
  }
  const lengths = cumulative(points);
  const total = lengths[lengths.length - 1] ?? 0;
  const layer = clampLayer(roads.layer(road));
  const climbs = ramps(roads, road, total).map((ramp) => rampBadgeAt(points, lengths, ramp));
  return [...climbs, ...levelBadges(points, lengths, layer)];
}

export function drawLevelBadges(
  pieces: PieceSet,
  roads: RoadStore,
  road: number,
  budget: BadgeBudget,
): void {
  const badges = roadBadges(roads, road).slice(0, Math.max(budget.remaining, 0));
  if (badges.length === 0) {
    return;
  }
  budget.remaining -= badges.length;
  const g = pieces.get(clampLayer(roads.layer(road)), 'markings');
  for (const badge of badges) {
    drawBadge(g, badge);
  }
}
