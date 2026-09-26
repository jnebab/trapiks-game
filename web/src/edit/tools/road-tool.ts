import type { QuoteOutcome } from '../../generated/QuoteOutcome';
import { costLabel } from '../../hud/inspector/view-models';
import type { RoadOptions } from '../../hud/road-options';
import type { RoadTooltip } from '../../hud/road-tooltip';
import { roadStyle } from '../../render/palette';
import type { Point } from '../../render/polyline';
import { screenToWorld } from '../../render/camera';
import type { CameraState } from '../../render/camera-input';
import type { TileManager } from '../../render/tiles/tile-manager';
import type { SimClient } from '../../sim/client';
import { trackPointer } from '../pointer-tracker';
import type { PreviewLayer } from './preview-layer';
import { planRoad, snapLookups, type Draft, type LookupSources } from './road-command';
import { RoadQuoter, type Planned } from './road-quote';
import { snapEndpoint, type Snapped } from './snap';

const ACTION = 'New road';
export const SNAP_MESSAGE = 'Snap to a road or junction';

export interface RoadToolContext extends LookupSources {
  canvas: HTMLCanvasElement;
  state: CameraState;
  tiles: TileManager;
  preview: PreviewLayer;
  tooltip: RoadTooltip;
  options: RoadOptions;
  client: Pick<SimClient, 'quote' | 'sendCommand'>;
  signal: AbortSignal;
}

export class RoadTool {
  active = false;
  locked = false;
  private draft: Draft | undefined;
  private planned: Planned | undefined;
  private readonly pointer: Point = { x: NaN, y: NaN };
  private readonly quoter: RoadQuoter;

  constructor(private readonly ctx: RoadToolContext) {
    this.quoter = new RoadQuoter(ctx.client, (planned, outcome) => {
      this.onQuote(planned, outcome);
    });
    const handlers = {
      onHover: (sx: number, sy: number) => {
        this.hover(sx, sy);
      },
      onClick: (sx: number, sy: number) => {
        this.click(sx, sy);
      },
      onPress: () => undefined,
    };
    trackPointer(ctx.canvas, handlers, ctx.signal);
    ctx.canvas.addEventListener('contextmenu', this.onContextMenu, { signal: ctx.signal });
  }

  setActive(active: boolean): void {
    this.active = active;
    this.cancel();
  }

  cancel(): boolean {
    const had = this.draft !== undefined;
    this.draft = undefined;
    this.planned = undefined;
    this.quoter.cancel();
    this.ctx.preview.clear();
    this.ctx.tooltip.hide();
    return had;
  }

  refresh(): void {
    this.quoter.forget();
    if (this.active && !Number.isNaN(this.pointer.x)) {
      this.hover(this.pointer.x, this.pointer.y);
    }
  }

  private readonly onContextMenu = (event: MouseEvent): void => {
    if (this.active) {
      event.preventDefault();
      this.cancel();
    }
  };

  private enabled(): boolean {
    return this.active && !this.locked && this.ctx.tiles.activeBand !== 'city';
  }

  private world(sx: number, sy: number): Point {
    const [x, y] = screenToWorld(this.ctx.state.camera, sx, sy);
    return { x, y };
  }

  private snap(point: Point): Snapped | null {
    const lookups = snapLookups(this.ctx, this.ctx.state.camera.scale);
    return snapEndpoint(point.x, point.y, lookups);
  }

  private hover(sx: number, sy: number): void {
    this.pointer.x = sx;
    this.pointer.y = sy;
    if (!this.enabled()) {
      return;
    }
    const point = this.world(sx, sy);
    const draft = this.draft;
    if (draft === undefined) {
      this.hoverStart(point);
    } else if (draft.placingVia) {
      this.ctx.preview.road([draft.start.point, point], this.width(), true);
    } else {
      this.hoverEnd(draft, point);
    }
  }

  private hoverStart(point: Point): void {
    const snapped = this.snap(point);
    if (snapped === null) {
      this.ctx.preview.clear();
    } else {
      this.ctx.preview.marker(snapped.point);
    }
  }

  private hoverEnd(draft: Draft, point: Point): void {
    const end = this.snap(point);
    if (end === null) {
      this.planned = undefined;
      this.quoter.cancel();
      this.ctx.preview.miss(draft.start.point, point);
      this.showTooltip(SNAP_MESSAGE);
      return;
    }
    const planned = planRoad(draft, end, this.ctx.options);
    this.planned = planned;
    const known = this.quoter.known(planned);
    this.ctx.preview.road(planned.points, this.width(), known === undefined || 'Ok' in known);
    if (known === undefined) {
      this.quoter.request(planned);
    } else {
      this.showTooltip(costLabel(known, ACTION));
    }
  }

  private onQuote(planned: Planned, outcome: QuoteOutcome): void {
    if (this.planned?.key !== planned.key) {
      return;
    }
    this.ctx.preview.road(planned.points, this.width(), 'Ok' in outcome);
    this.showTooltip(costLabel(outcome, ACTION));
  }

  private showTooltip(text: string): void {
    const rect = this.ctx.canvas.getBoundingClientRect();
    const { x, y } = this.pointer;
    this.ctx.tooltip.show(text, this.ctx.options.layer, rect.left + x, rect.top + y);
  }

  private width(): number {
    const [forward, backward] = this.ctx.options.lanes;
    return (forward + backward) * roadStyle.laneWidth;
  }

  private click(sx: number, sy: number): void {
    if (!this.enabled()) {
      return;
    }
    const point = this.world(sx, sy);
    const draft = this.draft;
    if (draft === undefined) {
      this.begin(point);
    } else if (draft.placingVia) {
      this.draft = { ...draft, via: point, placingVia: false };
    } else {
      this.finish(draft, point);
    }
    this.hover(sx, sy);
  }

  private begin(point: Point): void {
    const start = this.snap(point);
    if (start !== null) {
      this.draft = { start, via: undefined, placingVia: this.ctx.options.curve };
    }
  }

  private finish(draft: Draft, point: Point): void {
    const end = this.snap(point);
    if (end === null) {
      return;
    }
    const planned = planRoad(draft, end, this.ctx.options);
    this.planned = planned;
    const known = this.quoter.known(planned);
    if (known !== undefined) {
      this.commit(planned, known);
      return;
    }
    this.quoter.now(planned).then(
      (outcome) => {
        this.commit(planned, outcome);
      },
      () => undefined,
    );
  }

  private commit(planned: Planned, outcome: QuoteOutcome): void {
    if (!('Ok' in outcome) || this.planned?.key !== planned.key) {
      return;
    }
    this.ctx.client.sendCommand(planned.command);
    this.quoter.forget();
    this.cancel();
  }
}
