import { describe, expect, it } from 'vitest';
import { BufferPool, roundedCapacity } from './buffer-pool';

describe('BufferPool', () => {
  it('rounds new capacity up to 4096', () => {
    expect(roundedCapacity(1)).toBe(4096);
    expect(roundedCapacity(4096)).toBe(4096);
    expect(roundedCapacity(4097)).toBe(8192);
    const pool = new BufferPool();
    const set = pool.take(5000);
    expect(set.ids.length).toBe(8192);
    expect(set.style.length).toBe(8192);
  });

  it('reuses a returned set', () => {
    const pool = new BufferPool();
    const set = pool.take(10);
    pool.give(set);
    expect(pool.take(20)).toBe(set);
  });

  it('allocates when returned sets are too small', () => {
    const pool = new BufferPool();
    const small = pool.take(10);
    pool.give(small);
    const large = pool.take(5000);
    expect(large).not.toBe(small);
    expect(large.ids.length).toBe(8192);
  });

  it('drops sets smaller than the last take and keeps at most three', () => {
    const pool = new BufferPool();
    const small = pool.take(10);
    pool.take(9000);
    pool.give(small);
    expect(pool.size).toBe(0);
    const sets = [pool.take(9000), pool.take(9000), pool.take(9000), pool.take(9000)];
    sets.forEach((set) => {
      pool.give(set);
    });
    expect(pool.size).toBe(3);
  });
});
