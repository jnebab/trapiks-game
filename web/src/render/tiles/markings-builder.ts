import { clampLayer } from '../layer-range';
import type { RoadStore } from '../road-store';
import { drawMarker } from './marker-draw';
import { PieceSet, type TilePiece } from './piece-set';
import { drawRoadMarkings } from './road-markings';
import type { StreetData } from './street-data';
import type { TileEntry } from './tile-index';

function roadLayer(roads: RoadStore, road: number): number {
  return clampLayer(roads.layer(road));
}

export function buildMarkings(entry: TileEntry, roads: RoadStore, street: StreetData): TilePiece[] {
  const pieces = new PieceSet();
  for (const road of entry.roads) {
    if (!roads.isDeleted(road)) {
      drawRoadMarkings(pieces.get(roadLayer(roads, road), 'markings'), roads, street.detail, road);
    }
  }
  for (const id of entry.markers) {
    const marker = street.detail.markers.get(id);
    if (marker !== undefined) {
      const road = Math.floor(marker.link / 2);
      drawMarker(pieces.get(roadLayer(roads, road), 'markings'), marker);
    }
  }
  return pieces.list();
}
