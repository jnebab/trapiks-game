export function tileKey(band: string, tx: number, ty: number): string {
  return `${band}:${String(tx)}:${String(ty)}`;
}

export function tileOf(size: number, x: number, y: number): [number, number] {
  return [Math.floor(x / size), Math.floor(y / size)];
}
