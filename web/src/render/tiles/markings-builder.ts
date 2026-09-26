import type { RoadArrays } from '../../sim/protocol';
import { clampLayer } from '../layers';
import { drawMarker } from './marker-draw';
import { PieceSet, type TilePiece } from './piece-set';
import { drawRoadMarkings } from './road-markings';
import type { StreetData } from './street-data';
import type { TileEntry } from './tile-index';

function roadLayer(roads: RoadArrays, road: number): number {
  return clampLayer(roads.layer[road] ?? 0);
}

export function buildMarkings(
  entry: TileEntry,
  roads: RoadArrays,
  street: StreetData,
): TilePiece[] {
  const pieces = new PieceSet();
  for (const road of entry.roads) {
    drawRoadMarkings(pieces.get(roadLayer(roads, road), 'markings'), roads, street.setbacks, road);
  }
  for (const marker of entry.markers) {
    const road = Math.floor((street.markers.link[marker] ?? 0) / 2);
    drawMarker(pieces.get(roadLayer(roads, road), 'markings'), street.markers, marker);
  }
  return pieces.list();
}
