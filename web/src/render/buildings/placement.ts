import type { RoadClass } from '../../generated/RoadClass';
import type { RoadStore } from '../road-store';
import { laneWidth } from '../tiles/road-style';
import { buildingKind, hasBuildings, type BuildingKind } from './kinds';
import type { LotGrid } from './lot-grid';
import { corners, type Obb } from './obb';
import { lotSeed, mulberry32, type Rand } from './rng';
import { walkRoad, type Pose } from './road-walk';

export const MIN_ROAD_LENGTH = 40;
export const END_MARGIN = 12;
export const SETBACK = 3;
export const PARKING_DEPTH = 10;
const PARKING_SHARE = 0.3;

export interface Lot {
  kind: BuildingKind;
  body: Obb;
  parking: Obb | undefined;
}

export interface LotStore {
  roads: RoadStore;
  classNames: readonly RoadClass[];
  hitsRoad: (x: number, y: number) => boolean;
  nearJunction: (box: Obb) => boolean;
  grid: LotGrid;
}

const SIZES: Record<BuildingKind, readonly [number, number, number, number]> = {
  house: [8, 4, 8, 6],
  commercial: [14, 10, 12, 6],
  warehouse: [18, 10, 14, 8],
};

interface Frame {
  pose: Pose;
  nx: number;
  ny: number;
  edge: number;
}

function frameAt(pose: Pose, side: number, edge: number): Frame {
  const sign = side === 0 ? 1 : -1;
  return { pose, nx: -pose.ty * sign, ny: pose.tx * sign, edge };
}

function boxAt(frame: Frame, distance: number, halfW: number, halfD: number): Obb {
  const { pose, nx, ny } = frame;
  return {
    cx: pose.x + nx * distance,
    cy: pose.y + ny * distance,
    ux: pose.tx,
    uy: pose.ty,
    halfW,
    halfD,
  };
}

function makeLot(kind: BuildingKind, frame: Frame, rand: Rand): Lot {
  const [w0, w1, d0, d1] = SIZES[kind];
  const width = w0 + w1 * rand();
  const depth = d0 + d1 * rand();
  const near = frame.edge + SETBACK;
  const body = boxAt(frame, near + depth / 2, width / 2, depth / 2);
  const parked = kind === 'commercial' && rand() < PARKING_SHARE;
  const parking = parked
    ? boxAt(frame, near + depth + PARKING_DEPTH / 2, width / 2, PARKING_DEPTH / 2)
    : undefined;
  return { kind, body, parking };
}

function footprint(lot: Lot): Obb[] {
  return lot.parking === undefined ? [lot.body] : [lot.body, lot.parking];
}

function blocked(box: Obb, store: LotStore): boolean {
  const probes = [...corners(box), [box.cx, box.cy] as const];
  return (
    probes.some(([x, y]) => store.hitsRoad(x, y)) ||
    store.nearJunction(box) ||
    store.grid.collides(box)
  );
}

function accept(lot: Lot, store: LotStore): boolean {
  const boxes = footprint(lot);
  if (boxes.some((box) => blocked(box, store))) {
    return false;
  }
  boxes.forEach((box) => {
    store.grid.add(box);
  });
  return true;
}

export function placesBuildings(road: number, store: LotStore): boolean {
  const { roads } = store;
  return roads.layer(road) === 0 && hasBuildings(store.classNames[roads.classCode(road)]);
}

export function placeLots(road: number, side: number, store: LotStore): Lot[] {
  const walk = walkRoad(store.roads.pointsOf(road));
  if (!placesBuildings(road, store) || walk.length < MIN_ROAD_LENGTH) {
    return [];
  }
  const rand = mulberry32(lotSeed(road, side));
  const roadClass = store.classNames[store.roads.classCode(road)];
  const edge = laneWidth(store.roads, road) / 2;
  const lots: Lot[] = [];
  for (let s = END_MARGIN; s <= walk.length - END_MARGIN; s += 12 + 6 * rand()) {
    const kind = buildingKind(roadClass, rand) ?? 'house';
    const lot = makeLot(kind, frameAt(walk.pose(s), side, edge), rand);
    if (accept(lot, store)) {
      lots.push(lot);
    }
  }
  return lots;
}
