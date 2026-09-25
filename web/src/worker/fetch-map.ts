const GZIP_MAGIC = [0x1f, 0x8b] as const;

function isGzip(bytes: Uint8Array): boolean {
  return bytes[0] === GZIP_MAGIC[0] && bytes[1] === GZIP_MAGIC[1];
}

async function gunzip(bytes: Uint8Array<ArrayBuffer>): Promise<Uint8Array> {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip'));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

export async function fetchMap(url: string): Promise<Uint8Array> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${url}: ${String(response.status)}`);
  }
  const bytes = new Uint8Array(await response.arrayBuffer());
  return isGzip(bytes) ? gunzip(bytes) : bytes;
}
