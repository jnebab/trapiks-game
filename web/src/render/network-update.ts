import type { DeltaArrays } from '../sim/protocol';
import type { Picking } from '../edit/picking';
import type { DetailStore } from './detail-store';
import type { NodeStore } from './node-store';
import type { RoadRows, RoadStore } from './road-store';
import type { TileManager } from './tiles/tile-manager';

export interface NetworkTarget {
  roads: RoadStore;
  nodes: NodeStore;
  detail: DetailStore;
  tiles: TileManager;
  picking: Picking;
}

function roadRows(delta: DeltaArrays): RoadRows {
  return {
    ids: delta.roadIds,
    classCode: delta.roadClass,
    lanesForward: delta.roadLanesForward,
    lanesBackward: delta.roadLanesBackward,
    layer: delta.roadLayer,
    name: delta.roadName,
    roundabout: delta.roadRoundabout,
    deleted: delta.roadDeleted,
    from: delta.roadFrom,
    to: delta.roadTo,
    pointLen: delta.roadPointLen,
    x: delta.roadX,
    y: delta.roadY,
  };
}

function truncate(delta: DeltaArrays, target: NetworkTarget): void {
  const counts = { roads: delta.roadCount, nodes: delta.nodeCount };
  const markers = target.detail.truncateNodes(counts.nodes);
  target.roads.truncate(counts.roads);
  target.nodes.truncate(counts.nodes);
  target.picking.truncate(counts.roads);
  target.tiles.truncate(counts, markers.removed);
}

export function applyDelta(delta: DeltaArrays, target: NetworkTarget): void {
  truncate(delta, target);
  target.roads.applyRoads(roadRows(delta));
  target.nodes.applyNodes({
    ids: delta.nodeIds,
    x: delta.nodeX,
    y: delta.nodeY,
    control: delta.nodeControl,
  });
  target.detail.applySetbacks({
    ids: delta.setbackIds,
    start: delta.setbackStart,
    end: delta.setbackEnd,
  });
  target.detail.applyJunctions({
    node: delta.junctionNode,
    layer: delta.junctionLayer,
    minLayer: delta.junctionMinLayer,
    filletLayer: delta.junctionFilletLayer,
    ringStart: delta.junctionRingStart,
    x: delta.junctionX,
    y: delta.junctionY,
  });
  const markers = target.detail.replaceMarkers(delta.markerNodes, {
    link: delta.markerLink,
    node: delta.markerNode,
    kind: delta.markerKind,
    x1: delta.markerX1,
    y1: delta.markerY1,
    x2: delta.markerX2,
    y2: delta.markerY2,
  });
  target.tiles.invalidate({
    roads: Array.from(delta.roadIds),
    nodes: Array.from(delta.junctionNode),
    markers,
    setbackRoads: Array.from(delta.setbackIds),
  });
  target.picking.update(delta.roadIds);
}
