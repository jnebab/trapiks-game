import { readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';

const EARTH_RADIUS_M = 6_371_008.8;
const MAGIC = 'TRPK';
const ORIGIN_OFFSET = 8;

export interface GeoOrigin {
  lat: number;
  lon: number;
}

export interface MapPoint {
  x: number;
  y: number;
}

export function readMapOrigin(path: string): GeoOrigin {
  const bytes = gunzipSync(readFileSync(path));
  if (bytes.subarray(0, 4).toString('latin1') !== MAGIC) {
    throw new Error(`${path} is not a Trapiks map`);
  }
  return {
    lat: bytes.readDoubleLE(ORIGIN_OFFSET),
    lon: bytes.readDoubleLE(ORIGIN_OFFSET + 8),
  };
}

function radians(degrees: number): number {
  return (degrees * Math.PI) / 180;
}

export function projectLatLon(origin: GeoOrigin, lat: number, lon: number): MapPoint {
  const cosLat0 = Math.cos(radians(origin.lat));
  return {
    x: EARTH_RADIUS_M * radians(lon - origin.lon) * cosLat0,
    y: -EARTH_RADIUS_M * radians(lat - origin.lat),
  };
}
