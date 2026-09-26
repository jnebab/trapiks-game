import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { SaveStore, saveKey, type SaveRecord } from './saves';

class MemoryStorage implements Storage {
  private readonly items = new Map<string, string>();

  get length(): number {
    return this.items.size;
  }

  clear(): void {
    this.items.clear();
  }

  getItem(key: string): string | null {
    return this.items.get(key) ?? null;
  }

  key(index: number): string | null {
    return [...this.items.keys()][index] ?? null;
  }

  removeItem(key: string): void {
    this.items.delete(key);
  }

  setItem(key: string, value: string): void {
    this.items.set(key, value);
  }
}

function throwing(): Storage {
  throw new Error('storage disabled');
}

const HASH = 'abc123';

function record(overrides: Partial<SaveRecord> = {}): Omit<SaveRecord, 'bestStars'> {
  return {
    mapHash: HASH,
    mode: 'challenge:downtown',
    seed: 1,
    log: [{ DeleteRoad: { road: 4 } }],
    updatedAt: 10,
    ...overrides,
  };
}

describe('SaveStore', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('round trips a save after the debounce', () => {
    const storage = new MemoryStorage();
    const saves = new SaveStore(() => storage);
    saves.schedule(record());
    expect(saves.read(HASH, 'challenge:downtown')).toEqual({ kind: 'none' });
    vi.advanceTimersByTime(1000);
    expect(saves.read(HASH, 'challenge:downtown')).toEqual({ kind: 'found', save: record() });
  });

  it('flushes a pending write synchronously', () => {
    const storage = new MemoryStorage();
    const saves = new SaveStore(() => storage);
    saves.schedule(record({ mode: 'sandbox', vehiclesPerHour: 5000 }));
    saves.flush();
    const lookup = saves.read(HASH, 'sandbox');
    expect(lookup.kind === 'found' && lookup.save.vehiclesPerHour).toBe(5000);
  });

  it('ignores a save written for another map', () => {
    const storage = new MemoryStorage();
    const foreign = { ...record(), mapHash: 'other' };
    storage.setItem(saveKey(HASH, 'challenge:downtown'), JSON.stringify(foreign));
    const saves = new SaveStore(() => storage);
    expect(saves.read(HASH, 'challenge:downtown')).toEqual({ kind: 'otherMap' });
    expect(saves.latest(HASH)).toBeUndefined();
  });
});

describe('SaveStore stars and recency', () => {
  it('keeps the best stars at the maximum', () => {
    const storage = new MemoryStorage();
    const saves = new SaveStore(() => storage);
    saves.schedule(record());
    saves.recordStars(HASH, 'challenge:downtown', 2);
    saves.recordStars(HASH, 'challenge:downtown', 1);
    expect(saves.bestStars(HASH, 'challenge:downtown')).toBe(2);
    saves.schedule(record({ updatedAt: 20 }));
    saves.flush();
    expect(saves.bestStars(HASH, 'challenge:downtown')).toBe(2);
    saves.recordStars(HASH, 'challenge:downtown', 3);
    expect(saves.bestStars(HASH, 'challenge:downtown')).toBe(3);
  });

  it('picks the most recent save for the map', () => {
    const storage = new MemoryStorage();
    const saves = new SaveStore(() => storage);
    saves.schedule(record({ updatedAt: 5 }));
    saves.flush();
    saves.schedule(record({ mode: 'sandbox', updatedAt: 9 }));
    saves.flush();
    expect(saves.latest(HASH)?.mode).toBe('sandbox');
    saves.clear(HASH, 'sandbox');
    expect(saves.latest(HASH)?.mode).toBe('challenge:downtown');
  });
});

describe('SaveStore without storage', () => {
  it('works without storage', () => {
    const saves = new SaveStore(throwing);
    saves.schedule(record());
    expect(() => {
      saves.flush();
      saves.recordStars(HASH, 'challenge:downtown', 3);
      saves.clear(HASH, 'sandbox');
    }).not.toThrow();
    expect(saves.read(HASH, 'challenge:downtown')).toEqual({ kind: 'none' });
    expect(saves.latest(HASH)).toBeUndefined();
  });
});
