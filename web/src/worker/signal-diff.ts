export function statesChanged(previous: Uint8Array | undefined, next: Uint8Array): boolean {
  if (previous?.length !== next.length) {
    return true;
  }
  return next.some((state, i) => state !== previous[i]);
}
