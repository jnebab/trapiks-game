import type { EditCommand } from '../../generated/EditCommand';
import type { Endpoint } from '../../generated/Endpoint';
import type { RoadOptions } from '../../hud/road-options';
import type { NodeStore } from '../../render/node-store';
import type { Point } from '../../render/polyline';
import type { RoadStore } from '../../render/road-store';
import type { Picking } from '../picking';
import { roadGeometry } from './road-preview';
import type { Planned } from './road-quote';
import type { SnapLookups, Snapped } from './snap';

export function addRoadCommand(
  ends: readonly [Endpoint, Endpoint],
  via: Point | undefined,
  options: RoadOptions,
): EditCommand {
  return {
    AddRoad: {
      from: ends[0],
      to: ends[1],
      via: via === undefined ? null : [via.x, via.y],
      lanes_forward: options.lanes[0],
      lanes_backward: options.lanes[1],
      layer: options.layer,
    },
  };
}

export interface LookupSources {
  picking: Picking;
  roads: RoadStore;
  nodes: NodeStore;
}

export function snapLookups(sources: LookupSources, scale: number): SnapLookups {
  const { picking, roads, nodes } = sources;
  return {
    metresPerPixel: 1 / scale,
    pickNode: (x, y, radius) => picking.pickNode(x, y, radius),
    pickRoad: (x, y, tolerance) => picking.pickRoad(x, y, tolerance),
    roadPoints: (road) => roads.pointsOf(road),
    roadEnds: (road) => [roads.from(road), roads.to(road)],
    nodePoint: (node) => ({ x: nodes.x(node), y: nodes.y(node) }),
  };
}

export interface Draft {
  start: Snapped;
  via: Point | undefined;
  placingVia: boolean;
}

export function planRoad(draft: Draft, end: Snapped, options: RoadOptions): Planned {
  const ends = [draft.start.endpoint, end.endpoint] as const;
  const command = addRoadCommand(ends, draft.via, options);
  const points = roadGeometry(draft.start.point, end.point, draft.via);
  return { key: JSON.stringify(command), command, points };
}
