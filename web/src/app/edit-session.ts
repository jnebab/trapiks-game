import type { DebugApp } from '../render/app';
import type { CameraState } from '../render/camera-input';
import type { MapView } from '../render/map-view';
import { applyDelta } from '../render/network-update';
import { NodeHighlight } from '../render/node-highlight';
import { SelectionLayer } from '../render/selection';
import type { SignalPillLayer } from '../render/signal-pills';
import { EditController, type EditContext } from '../edit/edit-controller';
import { InspectSelection } from '../edit/inspect-selection';
import type { Picking } from '../edit/picking';
import type { MapMeta } from '../generated/MapMeta';
import { createInspectorPanel } from '../hud/inspector/panel';
import { createTooltip } from '../hud/inspector/tooltip';
import { createUndoChip, type UndoChip } from '../hud/inspector/undo-chip';
import type { CommandResultsMessage } from '../sim/protocol';
import type { SimClient } from '../sim/client';
import type { Lifetime } from './lifetime';
import { startRoadEditing, type RoadEditing } from './road-editing';

export interface EditHost {
  root: HTMLElement;
  slot: HTMLElement;
}

export interface EditTargets {
  view: MapView;
  camera: CameraState;
  client: SimClient;
  pills: SignalPillLayer;
  meta: MapMeta;
}

export interface EditSession {
  onResults: (message: CommandResultsMessage) => void;
  setLocked: (locked: boolean) => void;
}

function pillArrays(delta: NonNullable<CommandResultsMessage['delta']>) {
  return { link: delta.pillLink, x: delta.pillX, y: delta.pillY, angle: delta.pillAngle };
}

function createPanel(host: EditHost, targets: EditTargets, lifetime: Lifetime) {
  const { view, client, meta } = targets;
  const layer = document.createElement('div');
  host.root.appendChild(layer);
  lifetime.onDispose(() => {
    layer.remove();
  });
  const tooltip = createTooltip(layer);
  const stores = { roads: view.roads, names: meta.names, classNames: meta.class_names };
  const panel = createInspectorPanel({ client, tooltip, stores });
  layer.appendChild(panel.element);
  return { panel, tooltip };
}

interface ResultTargets {
  view: MapView;
  picking: Picking;
  selection: SelectionLayer;
  controller: EditController;
  pills: SignalPillLayer;
  undo: UndoChip;
  road: RoadEditing;
}

function resultsHandler(targets: ResultTargets) {
  const { view, picking, selection, controller, pills, undo, road } = targets;
  return ({ results, delta, budget }: CommandResultsMessage): void => {
    if (delta !== null) {
      applyDelta(delta, { ...view, picking });
      selection.refresh(new Set(delta.roadIds));
    }
    controller.onResults(results, budget);
    undo.refresh();
    road.tool.refresh();
    if (delta !== null) {
      pills.rebuild(pillArrays(delta));
    }
  };
}

function mountChips(host: EditHost, lifetime: Lifetime): HTMLElement {
  const chips = document.createElement('div');
  chips.className = 'chip-group';
  host.slot.appendChild(chips);
  lifetime.onDispose(() => {
    chips.remove();
  });
  return chips;
}

export function startEditing(
  scene: DebugApp,
  host: EditHost,
  targets: EditTargets,
  lifetime: Lifetime,
): EditSession {
  const { view, client, pills, camera } = targets;
  const picking = view.picking;
  const selection = new SelectionLayer(view.layers.selection, view.roads);
  lifetime.onTick(scene.app.ticker, () => {
    selection.setScale(camera.camera.scale);
  });
  const { panel, tooltip } = createPanel(host, targets, lifetime);
  const node = new NodeHighlight(view.layers.selection, view.detail);
  const inspect = new InspectSelection({
    client,
    panel,
    roads: selection,
    node,
    detail: view.detail,
  });
  const chips = mountChips(host, lifetime);
  const controller = createController(scene, host, targets, {
    picking,
    selection,
    inspect,
    topBar: chips,
    signal: lifetime.signal,
  });
  const undo = createUndoChip(client, tooltip);
  chips.appendChild(undo.element);
  const road = startRoadEditing(scene, host, { ...targets, picking, controller }, lifetime);
  return {
    onResults: resultsHandler({ view, picking, selection, controller, pills, undo, road }),
    setLocked: locker(controller, undo, inspect, road),
  };
}

type ControllerParts = Pick<EditContext, 'picking' | 'selection' | 'inspect' | 'topBar' | 'signal'>;

function createController(
  scene: DebugApp,
  host: EditHost,
  targets: EditTargets,
  parts: ControllerParts,
): EditController {
  return new EditController({
    ...parts,
    root: host.root,
    canvas: scene.app.canvas,
    state: targets.camera,
    tiles: targets.view.tiles,
    roads: targets.view.roads,
    send: targets.client.sendCommand,
  });
}

function locker(
  controller: EditController,
  undo: UndoChip,
  inspect: InspectSelection,
  road: RoadEditing,
) {
  return (locked: boolean): void => {
    controller.locked = locked;
    road.tools.setLocked(locked);
    undo.element.disabled = locked;
    if (locked) {
      inspect.select(undefined);
    }
  };
}
