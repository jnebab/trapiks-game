import type { EditCommand } from '../generated/EditCommand';
import { isEditCommands } from '../sim/game-values';
import { isRecord } from '../sim/values';

const PREFIX = 'trapiks:v1:';
const WRITE_DELAY_MS = 1000;

export type SaveMode = 'sandbox' | `challenge:${string}`;

export interface SaveRecord {
  mapHash: string;
  mode: SaveMode;
  seed: number;
  log: EditCommand[];
  bestStars?: number;
  vehiclesPerHour?: number;
  updatedAt: number;
}

export type SaveLookup =
  { kind: 'none' } | { kind: 'found'; save: SaveRecord } | { kind: 'otherMap' };

export function saveKey(mapHash: string, mode: SaveMode): string {
  return `${PREFIX}${mapHash}:${mode}`;
}

function isSaveMode(value: unknown): value is SaveMode {
  return value === 'sandbox' || (typeof value === 'string' && value.startsWith('challenge:'));
}

function optionalNumber(value: unknown): boolean {
  return value === undefined || typeof value === 'number';
}

export function isSaveRecord(value: unknown): value is SaveRecord {
  if (!isRecord(value)) {
    return false;
  }
  return (
    typeof value.mapHash === 'string' &&
    isSaveMode(value.mode) &&
    typeof value.seed === 'number' &&
    typeof value.updatedAt === 'number' &&
    isEditCommands(value.log) &&
    optionalNumber(value.bestStars) &&
    optionalNumber(value.vehiclesPerHour)
  );
}

function parse(raw: string | null): SaveRecord | undefined {
  if (raw === null) {
    return undefined;
  }
  try {
    const value: unknown = JSON.parse(raw);
    return isSaveRecord(value) ? value : undefined;
  } catch {
    return undefined;
  }
}

function safely<T>(action: () => T, fallback: T): T {
  try {
    return action();
  } catch {
    return fallback;
  }
}

export class SaveStore {
  private pending: SaveRecord | undefined;
  private timer: ReturnType<typeof setTimeout> | undefined;

  constructor(private readonly storage: () => Storage) {}

  read(mapHash: string, mode: SaveMode): SaveLookup {
    const save = parse(safely(() => this.storage().getItem(saveKey(mapHash, mode)), null));
    if (save === undefined) {
      return { kind: 'none' };
    }
    return save.mapHash === mapHash ? { kind: 'found', save } : { kind: 'otherMap' };
  }

  all(mapHash: string): SaveRecord[] {
    return safely(() => this.scan(), []).filter((save) => save.mapHash === mapHash);
  }

  latest(mapHash: string): SaveRecord | undefined {
    const saves = this.all(mapHash);
    return saves.reduce<SaveRecord | undefined>(
      (best, save) => (best === undefined || save.updatedAt > best.updatedAt ? save : best),
      undefined,
    );
  }

  bestStars(mapHash: string, mode: SaveMode): number | undefined {
    const lookup = this.read(mapHash, mode);
    return lookup.kind === 'found' ? lookup.save.bestStars : undefined;
  }

  schedule(record: Omit<SaveRecord, 'bestStars'>): void {
    this.pending = { ...record, ...this.kept(record.mapHash, record.mode) };
    clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.flush();
    }, WRITE_DELAY_MS);
  }

  flush(): void {
    clearTimeout(this.timer);
    const record = this.pending;
    this.pending = undefined;
    if (record !== undefined) {
      this.write(record);
    }
  }

  recordStars(mapHash: string, mode: SaveMode, stars: number): void {
    this.flush();
    const lookup = this.read(mapHash, mode);
    if (lookup.kind !== 'found') {
      return;
    }
    const best = Math.max(lookup.save.bestStars ?? 0, stars);
    this.write({ ...lookup.save, bestStars: best });
  }

  clear(mapHash: string, mode: SaveMode): void {
    if (this.pending?.mode === mode) {
      clearTimeout(this.timer);
      this.pending = undefined;
    }
    safely(() => {
      this.storage().removeItem(saveKey(mapHash, mode));
    }, undefined);
  }

  private kept(mapHash: string, mode: SaveMode): Pick<SaveRecord, 'bestStars'> {
    const stars = this.bestStars(mapHash, mode);
    return stars === undefined ? {} : { bestStars: stars };
  }

  private write(record: SaveRecord): void {
    safely(() => {
      this.storage().setItem(saveKey(record.mapHash, record.mode), JSON.stringify(record));
    }, undefined);
  }

  private scan(): SaveRecord[] {
    const storage = this.storage();
    const saves: SaveRecord[] = [];
    for (let i = 0; i < storage.length; i += 1) {
      const key = storage.key(i);
      const save = key?.startsWith(PREFIX) === true ? parse(storage.getItem(key)) : undefined;
      if (save !== undefined) {
        saves.push(save);
      }
    }
    return saves;
  }
}

export function browserSaves(): SaveStore {
  return new SaveStore(() => window.localStorage);
}
