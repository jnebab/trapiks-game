import type { Engine } from '../wasm/pkg/trapiks_sim_wasm.js';
import type { AreaArrays, NodeArrays, RoadArrays } from '../sim/protocol';

export interface Geometry {
  roads: RoadArrays;
  nodes: NodeArrays;
  areas: AreaArrays;
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

export function collectGeometry(engine: Engine): Geometry {
  return { roads: roadArrays(engine), nodes: nodeArrays(engine), areas: areaArrays(engine) };
}

export function transferList(geometry: Geometry): Transferable[] {
  return [geometry.roads, geometry.nodes, geometry.areas].flatMap((group) =>
    Object.values(group).map((array: ArrayBufferView) => array.buffer as ArrayBuffer),
  );
}
