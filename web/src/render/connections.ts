import type { RoadStore } from './road-store';

export type RoadGraph = Pick<
  RoadStore,
  'from' | 'to' | 'lanesForward' | 'lanesBackward' | 'isDeleted' | 'roadsAt'
>;

export interface Connections {
  outgoing: number[];
  incoming: number[];
  nodes: number[];
}

export function canLeave(graph: RoadGraph, road: number, node: number): boolean {
  const forward = node === graph.to(road) && graph.lanesForward(road) > 0;
  return forward || (node === graph.from(road) && graph.lanesBackward(road) > 0);
}

export function canEnter(graph: RoadGraph, road: number, node: number): boolean {
  const forward = node === graph.from(road) && graph.lanesForward(road) > 0;
  return forward || (node === graph.to(road) && graph.lanesBackward(road) > 0);
}

function sorted(values: Iterable<number>): number[] {
  return [...new Set(values)].sort((a, b) => a - b);
}

function neighbours(graph: RoadGraph, road: number | undefined, node: number): number[] {
  return graph.roadsAt(node).filter((other) => other !== road && !graph.isDeleted(other));
}

interface Collector {
  outgoing: number[];
  incoming: number[];
  nodes: number[];
}

function collectAt(graph: RoadGraph, road: number | undefined, node: number, out: Collector) {
  const others = neighbours(graph, road, node);
  const feeds = road === undefined || canEnter(graph, road, node);
  const drains = road === undefined || canLeave(graph, road, node);
  const incoming = feeds ? others.filter((other) => canLeave(graph, other, node)) : [];
  const outgoing = drains ? others.filter((other) => canEnter(graph, other, node)) : [];
  out.incoming.push(...incoming);
  out.outgoing.push(...outgoing);
  if (incoming.length + outgoing.length > 0) {
    out.nodes.push(node);
  }
}

function finish(out: Collector): Connections {
  return {
    outgoing: sorted(out.outgoing),
    incoming: sorted(out.incoming),
    nodes: sorted(out.nodes),
  };
}

export function roadConnections(graph: RoadGraph, road: number): Connections {
  const out: Collector = { outgoing: [], incoming: [], nodes: [] };
  if (graph.isDeleted(road)) {
    return finish(out);
  }
  for (const node of new Set([graph.from(road), graph.to(road)])) {
    collectAt(graph, road, node, out);
  }
  return finish(out);
}

export function nodeConnections(graph: RoadGraph, node: number): Connections {
  const out: Collector = { outgoing: [], incoming: [], nodes: [] };
  collectAt(graph, undefined, node, out);
  return finish(out);
}
