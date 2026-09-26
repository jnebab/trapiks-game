import { Graphics, Rectangle, Texture, type Renderer } from 'pixi.js';
import { WINDSHIELD } from './palette';

const PX_PER_M = 10;
const RESOLUTION = 4;
const PADDING_PX = 2;
const BORDER_PX = 0.12 * PX_PER_M;
const CORNER_PX = 0.45 * PX_PER_M;
const CAR_BORDER = 0xcfcfcf;
const JEEPNEY_BODY = 0xd9d9d9;
const JEEPNEY_RED = 0xe5484d;
const JEEPNEY_YELLOW = 0xf2c94c;
const JEEPNEY_RACK = 0x8a8a8a;
const BUS_BODY = 0x4a7bd0;
const BUS_BORDER = 0x2f5aa0;
const BUS_WINDOW_COUNT = 7;

interface Box {
  x: number;
  y: number;
  length: number;
  width: number;
}

const CAR: Box = { x: 0, y: 0, length: 4.4 * PX_PER_M, width: 1.9 * PX_PER_M };
const JEEPNEY: Box = {
  x: 0,
  y: CAR.width + PADDING_PX,
  length: 6.5 * PX_PER_M,
  width: 2.1 * PX_PER_M,
};
const BUS: Box = {
  x: 0,
  y: JEEPNEY.y + JEEPNEY.width + PADDING_PX,
  length: 12 * PX_PER_M,
  width: 2.5 * PX_PER_M,
};
const ATLAS_WIDTH = BUS.length;
const ATLAS_HEIGHT = BUS.y + BUS.width;

export type VehicleAtlas = readonly Texture[];

function body(g: Graphics, box: Box, fill: number, border: number): void {
  const inset = BORDER_PX / 2;
  g.roundRect(
    box.x + inset,
    box.y + inset,
    box.length - BORDER_PX,
    box.width - BORDER_PX,
    CORNER_PX,
  )
    .fill(fill)
    .stroke({ width: BORDER_PX, color: border });
}

interface Bar {
  at: number;
  length: number;
  span: number;
}

function glass(g: Graphics, box: Box, bar: Bar): void {
  const width = box.width * bar.span;
  const x = box.x + box.length * bar.at - bar.length / 2;
  g.rect(x, box.y + (box.width - width) / 2, bar.length, width).fill(WINDSHIELD);
}

function sideStripe(g: Graphics, box: Box, offset: number, color: number): void {
  const height = 0.12 * PX_PER_M;
  const from = box.x + box.length * 0.08;
  const length = box.length * 0.8;
  g.rect(from, box.y + offset, length, height).fill(color);
  g.rect(from, box.y + box.width - offset - height, length, height).fill(color);
}

function drawCar(g: Graphics): void {
  body(g, CAR, 0xffffff, CAR_BORDER);
  glass(g, CAR, { at: 0.7, length: 0.35 * PX_PER_M, span: 0.8 });
  glass(g, CAR, { at: 0.22, length: 0.25 * PX_PER_M, span: 0.8 });
}

function drawJeepney(g: Graphics): void {
  body(g, JEEPNEY, JEEPNEY_BODY, CAR_BORDER);
  sideStripe(g, JEEPNEY, 0.2 * PX_PER_M, JEEPNEY_RED);
  sideStripe(g, JEEPNEY, 0.34 * PX_PER_M, JEEPNEY_YELLOW);
  glass(g, JEEPNEY, { at: 0.84, length: 0.35 * PX_PER_M, span: 0.8 });
  const rack = 0.1 * PX_PER_M;
  g.rect(
    JEEPNEY.x + JEEPNEY.length * 0.15,
    JEEPNEY.y + (JEEPNEY.width - rack) / 2,
    JEEPNEY.length * 0.6,
    rack,
  ).fill(JEEPNEY_RACK);
}

function drawBus(g: Graphics): void {
  body(g, BUS, BUS_BODY, BUS_BORDER);
  glass(g, BUS, { at: 0.95, length: 0.35 * PX_PER_M, span: 0.85 });
  const pane = 0.12 * PX_PER_M;
  const spacing = (BUS.length * 0.8) / BUS_WINDOW_COUNT;
  for (let i = 0; i < BUS_WINDOW_COUNT; i += 1) {
    const x = BUS.x + BUS.length * 0.08 + i * spacing;
    g.rect(x, BUS.y + 0.25 * PX_PER_M, spacing * 0.7, pane).fill(WINDSHIELD);
    g.rect(x, BUS.y + BUS.width - 0.25 * PX_PER_M - pane, spacing * 0.7, pane).fill(WINDSHIELD);
  }
}

function frameOf(atlas: Texture, box: Box): Texture {
  return new Texture({
    source: atlas.source,
    frame: new Rectangle(box.x, box.y, box.length, box.width),
  });
}

export function createVehicleAtlas(renderer: Renderer): VehicleAtlas {
  const target = new Graphics();
  drawCar(target);
  drawJeepney(target);
  drawBus(target);
  const atlas = renderer.generateTexture({
    target,
    resolution: RESOLUTION,
    antialias: true,
    frame: new Rectangle(0, 0, ATLAS_WIDTH, ATLAS_HEIGHT),
  });
  target.destroy();
  return [CAR, JEEPNEY, BUS].map((box) => frameOf(atlas, box));
}
