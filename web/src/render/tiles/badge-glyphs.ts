import type { Point } from '../polyline';

export type Glyph = readonly (readonly Point[])[];

export const GLYPH_WIDTH = 0.6;

function path(...coords: number[]): Point[] {
  const points: Point[] = [];
  for (let i = 0; i + 1 < coords.length; i += 2) {
    points.push({ x: coords[i] ?? 0, y: coords[i + 1] ?? 0 });
  }
  return points;
}

const GLYPHS: Readonly<Record<string, Glyph>> = {
  L: [path(0, 0, 0, 1, 0.6, 1)],
  '1': [path(0.1, 0.2, 0.3, 0, 0.3, 1)],
  '2': [path(0, 0, 0.6, 0, 0.6, 0.5, 0, 0.5, 0, 1, 0.6, 1)],
  '3': [path(0, 0, 0.6, 0, 0.6, 1, 0, 1), path(0, 0.5, 0.6, 0.5)],
  '4': [path(0, 0, 0, 0.5, 0.6, 0.5), path(0.6, 0, 0.6, 1)],
  '5': [path(0.6, 0, 0, 0, 0, 0.5, 0.6, 0.5, 0.6, 1, 0, 1)],
  '↑': [path(0.3, 1, 0.3, 0), path(0, 0.35, 0.3, 0, 0.6, 0.35)],
  '↓': [path(0.3, 0, 0.3, 1), path(0, 0.65, 0.3, 1, 0.6, 0.65)],
};

export function glyphOf(char: string): Glyph {
  return GLYPHS[char] ?? [];
}
