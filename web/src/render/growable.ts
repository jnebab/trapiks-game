export type Column = Uint8Array | Int8Array | Uint32Array | Float32Array;

type Make<T extends Column> = (length: number) => T;

export function grow<T extends Column>(column: T, needed: number, make: Make<T>): T {
  if (needed <= column.length) {
    return column;
  }
  const next = make(Math.max(needed, column.length * 2, 16));
  next.set(column);
  return next;
}

export const makeU8 = (length: number): Uint8Array<ArrayBuffer> => new Uint8Array(length);
export const makeI8 = (length: number): Int8Array<ArrayBuffer> => new Int8Array(length);
export const makeU32 = (length: number): Uint32Array<ArrayBuffer> => new Uint32Array(length);
export const makeF32 = (length: number): Float32Array<ArrayBuffer> => new Float32Array(length);
