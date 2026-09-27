import type { DebugApp } from '../render/app';
import type { CameraState } from '../render/camera-input';
import type { MapView } from '../render/map-view';
import { applyDelta } from '../render/network-update';
import type { SignalPillLayer } from '../render/signal-pills';
import { EditController, type EditContext } from '../edit/edit-controller';
import type { Picking } from '../edit/picking';
import type { MapMeta } from '../generated/MapMeta';
import { createUndoChip, type UndoChip } from '../hud/inspector/undo-chip';
import type { CommandResultsMessage } from '../sim/protocol';
import type { SimClient } from '../sim/client';
import { createHighlights, createInspect, levelsFor } from './edit-highlights';
import type { Lifetime } from './lifetime';
import type { ConnectionHighlight } from '../render/connection-highlight';
import type { SelectionLayer } from '../render/selection';
import type { InspectSelection } from '../edit/inspect-selection';
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
  connectionSummary: () => string;
}

function pillArrays(delta: NonNullable<CommandResultsMessage['delta']>) {
  return { link: delta.pillLink, x: delta.pillX, y: delta.pillY, angle: delta.pillAngle };
}

interface ResultTargets {
  view: MapView;
  picking: Picking;
  selection: SelectionLayer;
  connections: ConnectionHighlight;
  controller: EditController;
  pills: SignalPillLayer;
  undo: UndoChip;
  road: RoadEditing;
}

function resultsHandler(targets: ResultTargets) {
  const { view, picking, selection, connections, controller, pills, undo, road } = targets;
  return ({ results, delta, budget }: CommandResultsMessage): void => {
    if (delta !== null) {
      applyDelta(delta, { ...view, picking });
      selection.refresh(new Set(delta.roadIds));
      connections.draw();
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
  const { view, client, pills } = targets;
  const picking = view.picking;
  const { selection, connections } = createHighlights(scene, targets, lifetime);
  const { inspect, tooltip } = createInspect(host, targets, { selection, connections }, lifetime);
  const chips = mountChips(host, lifetime);
  const controller = createController(scene, host, targets, {
    picking,
    selection,
    connections,
    inspect,
    topBar: chips,
    signal: lifetime.signal,
  });
  host.root.appendChild(levelsFor(view, controller, connections, lifetime.signal));
  const undo = createUndoChip(client, tooltip);
  chips.appendChild(undo.element);
  const road = startRoadEditing(scene, host, { ...targets, picking, controller }, lifetime);
  return {
    onResults: resultsHandler({
      view,
      picking,
      selection,
      connections,
      controller,
      pills,
      undo,
      road,
    }),
    setLocked: locker(controller, undo, inspect, road),
    connectionSummary: () => connections.summary(),
  };
}

type ControllerParts = Pick<
  EditContext,
  'picking' | 'selection' | 'connections' | 'inspect' | 'topBar' | 'signal'
>;

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
    levelView: targets.view.layers.levelView,
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
