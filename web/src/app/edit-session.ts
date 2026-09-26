import type { DebugApp } from '../render/app';
import type { MapView } from '../render/map-view';
import { applyDelta } from '../render/network-update';
import { NodeHighlight } from '../render/node-highlight';
import { SelectionLayer } from '../render/selection';
import type { SignalPillLayer } from '../render/signal-pills';
import { EditController } from '../edit/edit-controller';
import { InspectSelection } from '../edit/inspect-selection';
import { Picking } from '../edit/picking';
import type { MapMeta } from '../generated/MapMeta';
import { el } from '../hud/dom';
import { createInspectorPanel } from '../hud/inspector/panel';
import { createTooltip } from '../hud/inspector/tooltip';
import { createUndoChip } from '../hud/inspector/undo-chip';
import type { CommandResultsMessage } from '../sim/protocol';
import type { SimClient } from '../sim/client';

export type ResultsHandler = (message: CommandResultsMessage) => void;

function pillArrays(delta: NonNullable<CommandResultsMessage['delta']>) {
  return { link: delta.pillLink, x: delta.pillX, y: delta.pillY, angle: delta.pillAngle };
}

export interface EditTargets {
  view: MapView;
  client: SimClient;
  pills: SignalPillLayer;
  meta: MapMeta;
}

function createHud(root: HTMLElement, targets: EditTargets) {
  const { view, client, meta } = targets;
  const tooltip = createTooltip(root);
  const stores = { roads: view.roads, names: meta.names, classNames: meta.class_names };
  const panel = createInspectorPanel({ client, tooltip, stores });
  const undo = createUndoChip(client, tooltip);
  const topBar = el('div', 'top-bar');
  topBar.appendChild(undo.element);
  root.append(topBar, panel.element);
  return { panel, undo, topBar };
}

interface ResultTargets {
  view: MapView;
  picking: Picking;
  selection: SelectionLayer;
  controller: EditController;
  pills: SignalPillLayer;
}

function resultsHandler(targets: ResultTargets, after: () => void): ResultsHandler {
  const { view, picking, selection, controller, pills } = targets;
  return ({ results, delta, budget }) => {
    if (delta !== null) {
      applyDelta(delta, { ...view, picking });
      selection.refresh(new Set(delta.roadIds));
    }
    controller.onResults(results, budget);
    after();
    if (delta !== null) {
      pills.rebuild(pillArrays(delta));
    }
  };
}

export function startEditing(
  scene: DebugApp,
  root: HTMLElement,
  targets: EditTargets,
): ResultsHandler {
  const { view, client, pills } = targets;
  const picking = new Picking(view.roads, view.nodes, view.detail);
  const selection = new SelectionLayer(view.layers.selection, view.roads);
  scene.app.ticker.add(() => {
    selection.setScale(view.state.camera.scale);
  });
  const hud = createHud(root, targets);
  const inspect = new InspectSelection({
    client,
    panel: hud.panel,
    roads: selection,
    node: new NodeHighlight(view.layers.selection, view.detail),
    detail: view.detail,
  });
  const controller = new EditController({
    root,
    canvas: scene.app.canvas,
    state: view.state,
    tiles: view.tiles,
    picking,
    selection,
    roads: view.roads,
    inspect,
    topBar: hud.topBar,
    send: client.sendCommand,
  });
  const afterResults = (): void => {
    hud.undo.refresh();
  };
  return resultsHandler({ view, picking, selection, controller, pills }, afterResults);
}
