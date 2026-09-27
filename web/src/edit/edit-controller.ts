import type { BudgetState } from '../generated/BudgetState';
import type { CommandResult } from '../generated/CommandResult';
import type { EditCommand } from '../generated/EditCommand';
import { createBudgetChip, type BudgetChip } from '../hud/budget-chip';
import { isEditableTarget } from '../hud/keys';
import { errorMessage } from '../hud/messages';
import { showToast } from '../hud/toast';
import { screenToWorld } from '../render/camera';
import type { CameraState } from '../render/camera-input';
import type { ConnectionHighlight } from '../render/connection-highlight';
import { pickRank, type LevelView } from '../render/level-view';
import type { RoadStore } from '../render/road-store';
import type { SelectionLayer } from '../render/selection';
import type { TileManager } from '../render/tiles/tile-manager';
import type { InspectTarget } from '../sim/protocol';
import type { InspectSelection } from './inspect-selection';
import type { Picking } from './picking';
import { trackPointer, type ScreenPosition } from './pointer-tracker';

const PICK_TOLERANCE_PX = 6;
const NODE_PICK_PX = 10;
const MAX_TOLERANCE_M = 12;

export interface EditContext {
  root: HTMLElement;
  canvas: HTMLCanvasElement;
  state: CameraState;
  tiles: TileManager;
  picking: Picking;
  selection: SelectionLayer;
  connections: ConnectionHighlight;
  levelView: () => LevelView;
  roads: RoadStore;
  inspect: InspectSelection;
  topBar: HTMLElement;
  signal: AbortSignal;
  send: (command: EditCommand) => void;
}

function isUndoKey(event: KeyboardEvent): boolean {
  return (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'z';
}

export class EditController {
  locked = false;
  private selectEnabled = true;
  private readonly budget: BudgetChip = createBudgetChip();
  private readonly pointer: ScreenPosition;

  constructor(private readonly ctx: EditContext) {
    ctx.topBar.appendChild(this.budget.element);
    this.pointer = trackPointer(
      ctx.canvas,
      {
        onHover: (sx, sy) => {
          this.hover(this.pick(sx, sy));
        },
        onClick: (sx, sy) => {
          if (!this.locked && this.selectEnabled) {
            ctx.inspect.select(this.pickTarget(sx, sy));
          }
        },
        onPress: () => {
          this.hover(undefined);
        },
      },
      ctx.signal,
    );
    window.addEventListener('keydown', this.onKey, { signal: ctx.signal });
  }

  setSelectEnabled(enabled: boolean): void {
    this.selectEnabled = enabled;
    if (!enabled) {
      this.hover(undefined);
      this.ctx.inspect.select(undefined);
    }
  }

  onResults(results: readonly CommandResult[], budget: BudgetState): void {
    const { selection, roads, inspect } = this.ctx;
    this.hover(this.pick(this.pointer.x, this.pointer.y));
    if (selection.selected !== undefined && roads.isDeleted(selection.selected)) {
      inspect.select(undefined);
    }
    inspect.onResults(results);
    this.budget.set(budget);
    for (const result of results) {
      if ('Err' in result.outcome) {
        showToast(this.ctx.root, errorMessage(result.outcome.Err));
      }
    }
  }

  refreshHover(): void {
    this.hover(this.pick(this.pointer.x, this.pointer.y));
  }

  private hover(road: number | undefined): void {
    this.ctx.selection.setHover(road);
    this.ctx.connections.setHover(road);
  }

  private canPick(sx: number): boolean {
    return this.selectEnabled && !Number.isNaN(sx) && this.ctx.tiles.activeBand !== 'city';
  }

  private pick(sx: number, sy: number): number | undefined {
    if (!this.canPick(sx)) {
      return undefined;
    }
    const camera = this.ctx.state.camera;
    const [x, y] = screenToWorld(camera, sx, sy);
    const tolerance = Math.min(PICK_TOLERANCE_PX / camera.scale, MAX_TOLERANCE_M);
    const view = this.ctx.levelView();
    return this.ctx.picking.pickRoad(x, y, tolerance, (layer) => pickRank(view, layer));
  }

  private pickTarget(sx: number, sy: number): InspectTarget | undefined {
    if (!this.canPick(sx)) {
      return undefined;
    }
    const camera = this.ctx.state.camera;
    const [x, y] = screenToWorld(camera, sx, sy);
    const node = this.ctx.picking.pickNode(x, y, NODE_PICK_PX / camera.scale);
    if (node !== undefined) {
      return { node };
    }
    const road = this.pick(sx, sy);
    return road === undefined ? undefined : { road };
  }

  private readonly onKey = (event: KeyboardEvent): void => {
    if (this.locked || isEditableTarget(event)) {
      return;
    }
    if (event.key === 'Escape') {
      this.ctx.inspect.select(undefined);
      return;
    }
    const command = this.commandFor(event);
    if (command === undefined) {
      return;
    }
    event.preventDefault();
    this.ctx.send(command);
  };

  private commandFor(event: KeyboardEvent): EditCommand | undefined {
    if (isUndoKey(event)) {
      return 'Undo';
    }
    if (!this.selectEnabled || (event.key !== 'Delete' && event.key !== 'Backspace')) {
      return undefined;
    }
    const road = this.ctx.selection.selected ?? this.ctx.selection.hovered;
    return road === undefined ? undefined : { DeleteRoad: { road } };
  }
}
