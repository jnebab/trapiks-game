import type { DetailStore } from '../render/detail-store';
import type { NodeStore } from '../render/node-store';
import type { RoadStore } from '../render/road-store';
import { laneWidth } from '../render/tiles/road-style';
import { segmentDistance } from './segment-distance';

export const PICK_CELL = 32;
const CELL_OFFSET = 1 << 15;
const CELL_SPAN = 1 << 16;

interface Candidate {
  road: number;
  rank: number;
  gap: number;
}

export type LayerRank = (layer: number) => number | undefined;

const byLayer: LayerRank = (layer) => layer;

function cellOf(value: number): number {
  return Math.floor(value / PICK_CELL);
}

function cellIndex(cx: number, cy: number): number {
  return (cx + CELL_OFFSET) * CELL_SPAN + (cy + CELL_OFFSET);
}

function withoutRoad(pairs: readonly number[], road: number): number[] {
  const kept: number[] = [];
  for (let i = 0; i < pairs.length; i += 2) {
    if (pairs[i] !== road) {
      kept.push(pairs[i] ?? 0, pairs[i + 1] ?? 0);
    }
  }
  return kept;
}

function better(a: Candidate, b: Candidate | undefined): boolean {
  if (b === undefined) {
    return true;
  }
  if (a.rank !== b.rank) {
    return a.rank > b.rank;
  }
  if (a.gap !== b.gap) {
    return a.gap < b.gap;
  }
  return a.road < b.road;
}

export class Picking {
  private readonly cells = new Map<number, number[]>();
  private readonly roadCells = new Map<number, number[]>();

  constructor(
    private readonly roads: RoadStore,
    private readonly nodes: NodeStore,
    private readonly detail: DetailStore,
  ) {
    for (let road = 0; road < roads.count; road += 1) {
      this.insert(road);
    }
  }

  update(roadIds: Iterable<number>): void {
    for (const road of roadIds) {
      this.removeRoad(road);
      this.insert(road);
    }
  }

  truncate(count: number): void {
    for (const road of [...this.roadCells.keys()].filter((id) => id >= count)) {
      this.removeRoad(road);
    }
  }

  pickRoad(x: number, y: number, tolerance: number, rank = byLayer): number | undefined {
    let best: Candidate | undefined;
    for (const [road, segment] of this.nearby(x, y, tolerance)) {
      const candidate = this.candidate(road, segment, { x, y }, rank);
      if (candidate !== undefined && candidate.gap <= tolerance && better(candidate, best)) {
        best = candidate;
      }
    }
    return best?.road;
  }

  pickNode(x: number, y: number, radius: number): number | undefined {
    let best: number | undefined;
    let bestDistance = Infinity;
    const nodes = [...this.detail.junctions.keys()].sort((a, b) => a - b);
    for (const node of nodes) {
      const distance = Math.hypot(this.nodes.x(node) - x, this.nodes.y(node) - y);
      if (distance <= radius && distance < bestDistance) {
        best = node;
        bestDistance = distance;
      }
    }
    return best;
  }

  private candidate(
    road: number,
    segment: number,
    at: { x: number; y: number },
    rank: LayerRank,
  ): Candidate | undefined {
    const layerRank = rank(this.roads.layer(road));
    if (layerRank === undefined) {
      return undefined;
    }
    const [start] = this.roads.pointRange(road);
    const i = start + segment;
    const a = [this.roads.xs[i] ?? 0, this.roads.ys[i] ?? 0] as const;
    const b = [this.roads.xs[i + 1] ?? 0, this.roads.ys[i + 1] ?? 0] as const;
    const gap = segmentDistance(at.x, at.y, a, b) - laneWidth(this.roads, road) / 2;
    return { road, rank: layerRank, gap };
  }

  private *nearby(x: number, y: number, radius: number): Generator<[number, number]> {
    const seen = new Set<string>();
    for (let cx = cellOf(x - radius); cx <= cellOf(x + radius); cx += 1) {
      for (let cy = cellOf(y - radius); cy <= cellOf(y + radius); cy += 1) {
        yield* this.cellPairs(cellIndex(cx, cy), seen);
      }
    }
  }

  private *cellPairs(cell: number, seen: Set<string>): Generator<[number, number]> {
    const pairs = this.cells.get(cell) ?? [];
    for (let i = 0; i < pairs.length; i += 2) {
      const road = pairs[i] ?? 0;
      const segment = pairs[i + 1] ?? 0;
      const key = `${String(road)}:${String(segment)}`;
      if (!seen.has(key) && !this.roads.isDeleted(road)) {
        seen.add(key);
        yield [road, segment];
      }
    }
  }

  private insert(road: number): void {
    if (this.roads.isDeleted(road)) {
      return;
    }
    const [start, end] = this.roads.pointRange(road);
    const margin = laneWidth(this.roads, road) / 2;
    const touched = new Set<number>();
    for (let i = start; i + 1 < end; i += 1) {
      this.insertSegment(road, i - start, margin, touched);
    }
    this.roadCells.set(road, [...touched]);
  }

  private insertSegment(road: number, segment: number, margin: number, touched: Set<number>): void {
    const [start] = this.roads.pointRange(road);
    const i = start + segment;
    const xs = [this.roads.xs[i] ?? 0, this.roads.xs[i + 1] ?? 0];
    const ys = [this.roads.ys[i] ?? 0, this.roads.ys[i + 1] ?? 0];
    const minY = cellOf(Math.min(...ys) - margin);
    const maxY = cellOf(Math.max(...ys) + margin);
    for (
      let cx = cellOf(Math.min(...xs) - margin);
      cx <= cellOf(Math.max(...xs) + margin);
      cx += 1
    ) {
      for (let cy = minY; cy <= maxY; cy += 1) {
        const cell = cellIndex(cx, cy);
        this.cellList(cell).push(road, segment);
        touched.add(cell);
      }
    }
  }

  private cellList(cell: number): number[] {
    const existing = this.cells.get(cell);
    if (existing !== undefined) {
      return existing;
    }
    const created: number[] = [];
    this.cells.set(cell, created);
    return created;
  }

  private removeRoad(road: number): void {
    for (const cell of this.roadCells.get(road) ?? []) {
      this.cells.set(cell, withoutRoad(this.cells.get(cell) ?? [], road));
    }
    this.roadCells.delete(road);
  }
}
