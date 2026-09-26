import type { DebugApp } from '../render/app';
import type { MapView } from '../render/map-view';
import { applyDelta } from '../render/network-update';
import { SelectionLayer } from '../render/selection';
import type { SignalPillLayer } from '../render/signal-pills';
import { EditController } from '../edit/edit-controller';
import { Picking } from '../edit/picking';
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
  const controller = new EditController({
    root,
    canvas: scene.app.canvas,
    state: view.state,
    tiles: view.tiles,
    picking,
    selection,
    roads: view.roads,
    send: client.sendCommand,
  });
  return ({ results, delta, budget }) => {
    if (delta !== null) {
      applyDelta(delta, { ...view, picking });
      selection.refresh(new Set(delta.roadIds));
    }
    controller.onResults(results, budget);
    if (delta !== null) {
      pills.rebuild(pillArrays(delta));
    }
  };
}
