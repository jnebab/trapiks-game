import type { Ticker, TickerCallback } from 'pixi.js';

export class Lifetime {
  private readonly controller = new AbortController();
  private readonly cleanups: (() => void)[] = [];

  get signal(): AbortSignal {
    return this.controller.signal;
  }

  onTick(ticker: Ticker, callback: TickerCallback<unknown>): void {
    ticker.add(callback);
    this.cleanups.push(() => {
      ticker.remove(callback);
    });
  }

  onDispose(cleanup: () => void): void {
    this.cleanups.push(cleanup);
  }

  dispose(): void {
    this.controller.abort();
    for (const cleanup of this.cleanups.splice(0).reverse()) {
      cleanup();
    }
  }
}
