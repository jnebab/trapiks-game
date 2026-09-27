import type { EditController } from '../edit/edit-controller';
import { InspectSelection } from '../edit/inspect-selection';
import { createInspectorPanel } from '../hud/inspector/panel';
import { createTooltip } from '../hud/inspector/tooltip';
import { createLevelsControl } from '../hud/levels-control';
import type { DebugApp } from '../render/app';
import { ConnectionHighlight } from '../render/connection-highlight';
import type { LevelView } from '../render/level-view';
import type { MapView } from '../render/map-view';
import { NodeHighlight } from '../render/node-highlight';
import { SelectionLayer } from '../render/selection';
import type { EditHost, EditTargets } from './edit-session';
import type { Lifetime } from './lifetime';

export function createHighlights(
  scene: DebugApp,
  targets: EditTargets,
  lifetime: Lifetime,
): Highlights {
  const { view, camera } = targets;
  const selection = new SelectionLayer(view.layers.selection, view.roads);
  const connections = new ConnectionHighlight(view.layers.selection, {
    roads: view.roads,
    nodes: view.nodes,
    levelView: view.layers.levelView,
  });
  lifetime.onTick(scene.app.ticker, () => {
    selection.setScale(camera.camera.scale);
    connections.setScale(camera.camera.scale);
  });
  return { selection, connections };
}

export interface Highlights {
  selection: SelectionLayer;
  connections: ConnectionHighlight;
}

export function createInspect(
  host: EditHost,
  targets: EditTargets,
  highlights: Highlights,
  lifetime: Lifetime,
) {
  const { view, client } = targets;
  const { panel, tooltip } = createPanel(host, targets, lifetime);
  const node = new NodeHighlight(view.layers.selection, view.detail);
  const inspect = new InspectSelection({
    client,
    panel,
    roads: highlights.selection,
    node,
    connections: highlights.connections,
    detail: view.detail,
  });
  return { inspect, tooltip };
}

export function levelsFor(
  view: MapView,
  controller: EditController,
  connections: ConnectionHighlight,
  signal: AbortSignal,
): HTMLElement {
  const apply = (level: LevelView): void => {
    view.layers.setLevelView(level);
    controller.refreshHover();
    connections.draw();
  };
  const dock = document.createElement('div');
  dock.className = 'levels-dock';
  dock.appendChild(createLevelsControl(apply, signal));
  signal.addEventListener('abort', () => {
    dock.remove();
  });
  return dock;
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
