import { FrameStats } from './frame-stats';

export function perfFrom(search: string): number {
  const raw = Number(new URLSearchParams(search).get('perfFrom'));
  return Number.isFinite(raw) ? raw : 0;
}

export class StepStats {
  private samples: number[] = [];

  constructor(private readonly fromTick: number) {}

  record(tick: number, stepMs: Float64Array): void {
    if (tick >= this.fromTick) {
      this.samples.push(...stepMs);
    }
  }

  summary(): Record<string, string> {
    const stats = new FrameStats(this.samples.length);
    this.samples.forEach((ms) => {
      stats.record(ms);
    });
    return {
      stepCount: String(this.samples.length),
      stepP50: stats.percentile(0.5).toFixed(2),
      stepP95: stats.percentile(0.95).toFixed(2),
      stepMax: stats.percentile(1).toFixed(2),
    };
  }
}
