import type { Graphics } from 'pixi.js';
import { corners, flat, type Obb } from './obb';
import type { Lot } from './placement';

export const BUILDING_COLORS = {
  shadow: 0x000000,
  house: 0x8fd16a,
  houseShade: 0x62a83e,
  commercial: 0x4f7fd8,
  commercialEdge: 0x6e97e3,
  warehouse: 0x5a5a5a,
  warehouseLine: 0x4a4a4a,
  parking: 0xffffff,
} as const;

const SHADOW_ALPHA = 0.16;
const SHADOW_OFFSET = 0.8;
const ROOF_EDGE = 0.6;
const ROOF_LINES = 3;
const ROOF_LINE_WIDTH = 0.25;
const PARKING_ALPHA = 0.35;
const STRIPE_WIDTH = 0.12;
const STRIPE_SPACING = 2.5;
const LIGHT: readonly [number, number] = [1, 1];

function shifted(box: Obb, dx: number, dy: number): Obb {
  return { ...box, cx: box.cx + dx, cy: box.cy + dy };
}

function inset(box: Obb, by: number): Obb {
  return { ...box, halfW: Math.max(box.halfW - by, 0), halfD: Math.max(box.halfD - by, 0) };
}

function fillBox(g: Graphics, box: Obb, color: number, alpha = 1): void {
  g.poly(flat(corners(box)), true).fill({ color, alpha });
}

function drawParking(g: Graphics, box: Obb): void {
  fillBox(g, box, BUILDING_COLORS.parking, PARKING_ALPHA);
  const count = Math.floor((box.halfW * 2) / STRIPE_SPACING);
  for (let i = 1; i < count; i += 1) {
    const along = -box.halfW + i * STRIPE_SPACING;
    const stripe = { ...box, cx: box.cx + box.ux * along, cy: box.cy + box.uy * along };
    fillBox(g, { ...stripe, halfW: STRIPE_WIDTH / 2 }, BUILDING_COLORS.parking);
  }
}

function longAxis(box: Obb): Obb {
  if (box.halfW >= box.halfD) {
    return box;
  }
  return { ...box, ux: -box.uy, uy: box.ux, halfW: box.halfD, halfD: box.halfW };
}

function drawHouse(g: Graphics, box: Obb): void {
  fillBox(g, box, BUILDING_COLORS.house);
  const long = longAxis(box);
  const nx = -long.uy;
  const ny = long.ux;
  const sign = nx * LIGHT[0] + ny * LIGHT[1] >= 0 ? 1 : -1;
  const half = long.halfD / 2;
  const shade = { ...long, cx: long.cx + nx * half * sign, cy: long.cy + ny * half * sign };
  fillBox(g, { ...shade, halfD: half }, BUILDING_COLORS.houseShade);
}

function drawCommercial(g: Graphics, box: Obb): void {
  fillBox(g, box, BUILDING_COLORS.commercialEdge);
  fillBox(g, inset(box, ROOF_EDGE), BUILDING_COLORS.commercial);
}

function drawWarehouse(g: Graphics, box: Obb): void {
  fillBox(g, box, BUILDING_COLORS.warehouse);
  const long = longAxis(box);
  for (let i = 1; i <= ROOF_LINES; i += 1) {
    const along = -long.halfW + (i * 2 * long.halfW) / (ROOF_LINES + 1);
    const line = { ...long, cx: long.cx + long.ux * along, cy: long.cy + long.uy * along };
    fillBox(g, { ...line, halfW: ROOF_LINE_WIDTH / 2 }, BUILDING_COLORS.warehouseLine);
  }
}

const BODIES = { house: drawHouse, commercial: drawCommercial, warehouse: drawWarehouse } as const;

export function drawBuildings(lots: readonly Lot[], g: Graphics): void {
  for (const lot of lots) {
    if (lot.parking !== undefined) {
      drawParking(g, lot.parking);
    }
  }
  for (const lot of lots) {
    fillBox(
      g,
      shifted(lot.body, SHADOW_OFFSET, SHADOW_OFFSET),
      BUILDING_COLORS.shadow,
      SHADOW_ALPHA,
    );
  }
  for (const lot of lots) {
    BODIES[lot.kind](g, lot.body);
  }
}
