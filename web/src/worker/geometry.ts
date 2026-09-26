import type { Engine } from '../wasm/pkg/trapiks_sim_wasm.js';
import type {
  ApproachMarkerArrays,
  AreaArrays,
  JunctionShapeArrays,
  NodeArrays,
  RoadArrays,
} from '../sim/protocol';

export interface Geometry {
  roads: RoadArrays;
  nodes: NodeArrays;
  areas: AreaArrays;
  roadSetbacks: Float32Array;
  junctions: JunctionShapeArrays;
  markers: ApproachMarkerArrays;
}

function roadArrays(engine: Engine): RoadArrays {
  const g = engine.roadGeometry();
  const roads: RoadArrays = {
    pointStart: g.pointStart,
    x: g.x,
    y: g.y,
    classCode: g.classCode,
    lanesForward: g.lanesForward,
    lanesBackward: g.lanesBackward,
    layer: g.layer,
    name: g.name,
  };
  g.free();
  return roads;
}

function nodeArrays(engine: Engine): NodeArrays {
  const g = engine.nodeGeometry();
  const nodes: NodeArrays = { x: g.x, y: g.y, controlCode: g.controlCode };
  g.free();
  return nodes;
}

function areaArrays(engine: Engine): AreaArrays {
  const g = engine.areaGeometry();
  const areas: AreaArrays = { kindCode: g.kindCode, ringStart: g.ringStart, x: g.x, y: g.y };
  g.free();
  return areas;
}

function junctionArrays(engine: Engine): JunctionShapeArrays {
  const g = engine.junctionShapes();
  const junctions: JunctionShapeArrays = {
    node: g.node,
    layer: g.layer,
    minLayer: g.minLayer,
    ringStart: g.ringStart,
    x: g.x,
    y: g.y,
  };
  g.free();
  return junctions;
}

function markerArrays(engine: Engine): ApproachMarkerArrays {
  const g = engine.approachMarkers();
  const markers: ApproachMarkerArrays = {
    link: g.link,
    node: g.node,
    kind: g.kind,
    x1: g.x1,
    y1: g.y1,
    x2: g.x2,
    y2: g.y2,
  };
  g.free();
  return markers;
}

export function collectGeometry(engine: Engine): Geometry {
  return {
    roads: roadArrays(engine),
    nodes: nodeArrays(engine),
    areas: areaArrays(engine),
    roadSetbacks: engine.roadSetbacks(),
    junctions: junctionArrays(engine),
    markers: markerArrays(engine),
  };
}

export function transferList(geometry: Geometry): Transferable[] {
  const groups = [
    geometry.roads,
    geometry.nodes,
    geometry.areas,
    geometry.junctions,
    geometry.markers,
    { roadSetbacks: geometry.roadSetbacks },
  ];
  return groups.flatMap((group) =>
    Object.values(group).map((array: ArrayBufferView) => array.buffer as ArrayBuffer),
  );
}
