import type { DebugApp } from '../render/app';
import { PreviewLayer } from '../edit/tools/preview-layer';
import { RoadTool } from '../edit/tools/road-tool';
import { createToolSwitch, type ToolSwitch } from '../edit/tools/tool-switch';
import type { EditController } from '../edit/edit-controller';
import type { Picking } from '../edit/picking';
import { createRoadOptions } from '../hud/road-options';
import { createRoadTooltip } from '../hud/road-tooltip';
import type { EditHost, EditTargets } from './edit-session';
import type { Lifetime } from './lifetime';

export interface RoadEditing {
  tool: RoadTool;
  tools: ToolSwitch;
}

function createOverlays(scene: DebugApp, host: EditHost, targets: EditTargets, lifetime: Lifetime) {
  const preview = new PreviewLayer(targets.view.layers.selection);
  lifetime.onTick(scene.app.ticker, () => {
    preview.setScale(targets.camera.camera.scale);
  });
  const tooltip = createRoadTooltip(host.root);
  lifetime.onDispose(() => {
    tooltip.element.remove();
  });
  return { preview, tooltip };
}

export function startRoadEditing(
  scene: DebugApp,
  host: EditHost,
  targets: EditTargets & { picking: Picking; controller: EditController },
  lifetime: Lifetime,
): RoadEditing {
  const { view, camera, client, picking, controller } = targets;
  const { preview, tooltip } = createOverlays(scene, host, targets, lifetime);
  const refresh = (): void => {
    tool.refresh();
  };
  const options = createRoadOptions(refresh);
  const tool = new RoadTool({
    canvas: scene.app.canvas,
    state: camera,
    tiles: view.tiles,
    preview,
    tooltip,
    options: options.options,
    client,
    signal: lifetime.signal,
    picking,
    roads: view.roads,
    nodes: view.nodes,
  });
  const tools = createToolSwitch({
    root: host.root,
    options,
    road: tool,
    setSelectEnabled: (enabled) => {
      controller.setSelectEnabled(enabled);
    },
    signal: lifetime.signal,
  });
  return { tool, tools };
}
