import type { QuoteOutcome } from '../generated/QuoteOutcome';
import type { Inspection } from './protocol';

interface Waiter<T> {
  resolve: (result: T) => void;
  reject: (reason: Error) => void;
}

export class Waiters<T> {
  private readonly waiting = new Map<number, Waiter<T>>();

  constructor(private readonly ids: { next: number }) {}

  request(send: (id: number) => void): Promise<T> {
    this.ids.next += 1;
    const id = this.ids.next;
    return new Promise((resolve, reject) => {
      this.waiting.set(id, { resolve, reject });
      send(id);
    });
  }

  settle(id: number, result: T): void {
    this.waiting.get(id)?.resolve(result);
    this.waiting.delete(id);
  }

  rejectAll(): void {
    for (const waiter of this.waiting.values()) {
      waiter.reject(new Error('Sim session changed'));
    }
    this.waiting.clear();
  }
}

export interface Pending {
  quotes: Waiters<QuoteOutcome>;
  inspections: Waiters<Inspection | null>;
  rejectAll: () => void;
}

export function createPending(): Pending {
  const ids = { next: 0 };
  const quotes = new Waiters<QuoteOutcome>(ids);
  const inspections = new Waiters<Inspection | null>(ids);
  return {
    quotes,
    inspections,
    rejectAll: () => {
      quotes.rejectAll();
      inspections.rejectAll();
    },
  };
}
