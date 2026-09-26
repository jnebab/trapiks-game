import { expand, intersects, type Rect } from '../rect';
import type { TileIndex } from './tile-index';

interface Candidate {
  key: string;
  distance: number;
}

function byDistanceThenKey(a: Candidate, b: Candidate): number {
  if (a.distance !== b.distance) {
    return a.distance - b.distance;
  }
  return a.key < b.key ? -1 : Number(a.key > b.key);
}

export function visibleTiles(index: TileIndex, view: Rect, margin: number): string[] {
  const area = expand(view, margin);
  const centerX = (view.minX + view.maxX) / 2;
  const centerY = (view.minY + view.maxY) / 2;
  const found: Candidate[] = [];
  for (const [key, entry] of index) {
    if (intersects(entry.bounds, area)) {
      found.push({ key, distance: Math.hypot(entry.cx - centerX, entry.cy - centerY) });
    }
  }
  return found.sort(byDistanceThenKey).map((candidate) => candidate.key);
}
