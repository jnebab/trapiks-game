import { afterEach, describe, expect, it, vi } from 'vitest';
import { fetchMap } from './fetch-map';

const plain: Uint8Array<ArrayBuffer> = new Uint8Array([84, 82, 80, 75, 2, 0, 0, 0, 9, 8, 7]);

async function gzip(bytes: Uint8Array<ArrayBuffer>): Promise<Uint8Array<ArrayBuffer>> {
  const stream = new Blob([bytes]).stream().pipeThrough(new CompressionStream('gzip'));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

function stubFetch(body: Uint8Array<ArrayBuffer>, status = 200): void {
  vi.stubGlobal('fetch', () => Promise.resolve(new Response(body, { status })));
}

describe('fetchMap', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('gunzips gzipped bytes', async () => {
    stubFetch(await gzip(plain));
    expect(await fetchMap('http://test/map.bin.gz')).toEqual(plain);
  });

  it('returns already-plain bytes unchanged', async () => {
    stubFetch(plain);
    expect(await fetchMap('http://test/map.bin.gz')).toEqual(plain);
  });

  it('throws on a non-OK status', async () => {
    stubFetch(new Uint8Array(0), 404);
    await expect(fetchMap('http://test/missing.bin.gz')).rejects.toThrow('404');
  });
});
