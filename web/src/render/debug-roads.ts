import { Graphics } from 'pixi.js';
import type { RoadArrays } from '../sim/protocol';
import { palette, roadStyle } from './palette';

type Groups = Map<number, number[]>;

function laneCount(roads: RoadArrays, road: number): number {
  const total = (roads.lanesForward[road] ?? 0) + (roads.lanesBackward[road] ?? 0);
  return Math.max(total, 1);
}

function groupRoads(roads: RoadArrays, elevated: boolean): Groups {
  const groups: Groups = new Map();
  for (let road = 0; road < roads.layer.length; road += 1) {
    if ((roads.layer[road] ?? 0) > 0 !== elevated) {
      continue;
    }
    const lanes = laneCount(roads, road);
    const group = groups.get(lanes) ?? [];
    group.push(road);
    groups.set(lanes, group);
  }
  return groups;
}

function tracePolyline(g: Graphics, roads: RoadArrays, road: number): void {
  const start = roads.pointStart[road] ?? 0;
  const end = roads.pointStart[road + 1] ?? start;
  g.moveTo(roads.x[start] ?? 0, roads.y[start] ?? 0);
  for (let i = start + 1; i < end; i += 1) {
    g.lineTo(roads.x[i] ?? 0, roads.y[i] ?? 0);
  }
}

interface Pass {
  color: number;
  extra: number;
}

function strokePass(g: Graphics, roads: RoadArrays, groups: Groups, pass: Pass): void {
  for (const [lanes, group] of groups) {
    for (const road of group) {
      tracePolyline(g, roads, road);
    }
    const width = lanes * roadStyle.laneWidth + pass.extra;
    g.stroke({ width, color: pass.color, join: 'round', cap: 'round' });
  }
}

function drawLayer(g: Graphics, roads: RoadArrays, elevated: boolean): void {
  const groups = groupRoads(roads, elevated);
  const outline = elevated ? palette.elevatedOutline : palette.roadOutline;
  strokePass(g, roads, groups, { color: outline, extra: roadStyle.outlineExtra });
  strokePass(g, roads, groups, { color: palette.roadFill, extra: 0 });
}

export function drawRoads(roads: RoadArrays): Graphics {
  const g = new Graphics();
  drawLayer(g, roads, false);
  drawLayer(g, roads, true);
  return g;
}
