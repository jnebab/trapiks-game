export const FRAME_WINDOW = 300;

export class FrameStats {
  private readonly samples: number[] = [];

  constructor(private readonly window = FRAME_WINDOW) {}

  record(ms: number): void {
    this.samples.push(ms);
    if (this.samples.length > this.window) {
      this.samples.shift();
    }
  }

  percentile(fraction: number): number {
    if (this.samples.length === 0) {
      return 0;
    }
    const sorted = [...this.samples].sort((a, b) => a - b);
    const index = Math.min(Math.floor(sorted.length * fraction), sorted.length - 1);
    return sorted[index] ?? 0;
  }
}

export function perfEnabled(search: string): boolean {
  return new URLSearchParams(search).get('perf') === '1';
}
