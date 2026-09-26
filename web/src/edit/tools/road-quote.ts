import type { EditCommand } from '../../generated/EditCommand';
import type { QuoteOutcome } from '../../generated/QuoteOutcome';
import type { Point } from '../../render/polyline';
import type { SimClient } from '../../sim/client';

const QUOTE_DEBOUNCE_MS = 80;

export interface Planned {
  key: string;
  command: EditCommand;
  points: Point[];
}

export type QuoteListener = (planned: Planned, outcome: QuoteOutcome) => void;

export class RoadQuoter {
  private quoted: { key: string; outcome: QuoteOutcome } | undefined;
  private timer: number | undefined;

  constructor(
    private readonly client: Pick<SimClient, 'quote'>,
    private readonly listener: QuoteListener,
  ) {}

  known(planned: Planned): QuoteOutcome | undefined {
    return this.quoted?.key === planned.key ? this.quoted.outcome : undefined;
  }

  request(planned: Planned): void {
    window.clearTimeout(this.timer);
    this.timer = window.setTimeout(() => {
      this.now(planned).then(
        () => undefined,
        () => undefined,
      );
    }, QUOTE_DEBOUNCE_MS);
  }

  async now(planned: Planned): Promise<QuoteOutcome> {
    const outcome = await this.client.quote(planned.command);
    this.quoted = { key: planned.key, outcome };
    this.listener(planned, outcome);
    return outcome;
  }

  cancel(): void {
    window.clearTimeout(this.timer);
  }

  forget(): void {
    this.cancel();
    this.quoted = undefined;
  }
}
