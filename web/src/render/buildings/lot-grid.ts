import { extent, overlaps, type Obb } from './obb';

export const LOT_CELL = 32;

function cellKey(cx: number, cy: number): string {
  return `${String(cx)},${String(cy)}`;
}

function cellRange(value: number, reach: number): [number, number] {
  return [Math.floor((value - reach) / LOT_CELL), Math.floor((value + reach) / LOT_CELL)];
}

export class LotGrid {
  private readonly cells = new Map<string, Obb[]>();

  collides(box: Obb): boolean {
    return this.keys(box).some((key) => (this.cells.get(key) ?? []).some((b) => overlaps(box, b)));
  }

  add(box: Obb): void {
    for (const key of this.keys(box)) {
      const list = this.cells.get(key) ?? [];
      list.push(box);
      this.cells.set(key, list);
    }
  }

  private keys(box: Obb): string[] {
    const reach = extent(box);
    const [x0, x1] = cellRange(box.cx, reach);
    const [y0, y1] = cellRange(box.cy, reach);
    const keys: string[] = [];
    for (let cx = x0; cx <= x1; cx += 1) {
      for (let cy = y0; cy <= y1; cy += 1) {
        keys.push(cellKey(cx, cy));
      }
    }
    return keys;
  }
}
