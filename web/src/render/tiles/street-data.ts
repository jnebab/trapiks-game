import type { DetailStore, JunctionRing, Marker } from '../detail-store';
import type { NodeStore } from '../node-store';
import type { Rect } from '../rect';
import type { RoadStore } from '../road-store';
import { roadBox } from './road-bounds';

export interface StreetData {
  detail: DetailStore;
  nodes: NodeStore;
}

export interface Owned {
  x: number;
  y: number;
  box: Rect;
}

function ringBox(ring: JunctionRing): Rect {
  const xs = ring.points.filter((_, i) => i % 2 === 0);
  const ys = ring.points.filter((_, i) => i % 2 === 1);
  return {
    minX: Math.min(...xs),
    minY: Math.min(...ys),
    maxX: Math.max(...xs),
    maxY: Math.max(...ys),
  };
}

export function roadOwner(roads: RoadStore, road: number): Owned {
  const box = roadBox(roads, road);
  return { x: (box.minX + box.maxX) / 2, y: (box.minY + box.maxY) / 2, box };
}

export function junctionOwner(nodes: NodeStore, ring: JunctionRing): Owned {
  return { x: nodes.x(ring.node), y: nodes.y(ring.node), box: ringBox(ring) };
}

export function markerOwner(m: Marker): Owned {
  return {
    x: (m.x1 + m.x2) / 2,
    y: (m.y1 + m.y2) / 2,
    box: {
      minX: Math.min(m.x1, m.x2),
      minY: Math.min(m.y1, m.y2),
      maxX: Math.max(m.x1, m.x2),
      maxY: Math.max(m.y1, m.y2),
    },
  };
}
