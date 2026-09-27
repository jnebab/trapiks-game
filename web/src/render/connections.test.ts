import { describe, expect, it } from 'vitest';
import { nodeConnections, roadConnections, type RoadGraph } from './connections';

interface Row {
  from: number;
  to: number;
  forward: number;
  backward: number;
}

function graphOf(rows: readonly Row[], deleted: ReadonlySet<number> = new Set()): RoadGraph {
  const row = (road: number): Row => rows[road] ?? { from: -1, to: -1, forward: 0, backward: 0 };
  return {
    from: (road) => row(road).from,
    to: (road) => row(road).to,
    lanesForward: (road) => row(road).forward,
    lanesBackward: (road) => row(road).backward,
    isDeleted: (road) => road >= rows.length || deleted.has(road),
    roadsAt: (node) => rows.flatMap((r, road) => (r.from === node || r.to === node ? [road] : [])),
  };
}

const ROADS: readonly Row[] = [
  { from: 0, to: 1, forward: 2, backward: 0 },
  { from: 1, to: 2, forward: 1, backward: 1 },
  { from: 3, to: 1, forward: 1, backward: 0 },
  { from: 2, to: 4, forward: 0, backward: 1 },
  { from: 5, to: 6, forward: 2, backward: 2 },
  { from: 1, to: 7, forward: 0, backward: 1 },
];

describe('roadConnections', () => {
  const graph = graphOf(ROADS);

  it('follows a one-way road only at its downstream end', () => {
    expect(roadConnections(graph, 0)).toEqual({ outgoing: [1], incoming: [], nodes: [1] });
  });

  it('uses both ends of a two-way road', () => {
    expect(roadConnections(graph, 1)).toEqual({
      outgoing: [],
      incoming: [0, 2, 3, 5],
      nodes: [1, 2],
    });
  });

  it('ignores roads without a shared node', () => {
    expect(roadConnections(graph, 4)).toEqual({ outgoing: [], incoming: [], nodes: [] });
  });

  it('skips deleted roads', () => {
    const cut = graphOf(ROADS, new Set([1]));
    expect(roadConnections(cut, 0).outgoing).toEqual([]);
  });
});

describe('nodeConnections', () => {
  it('lists every road entering and leaving a junction', () => {
    expect(nodeConnections(graphOf(ROADS), 1)).toEqual({
      outgoing: [1],
      incoming: [0, 1, 2, 5],
      nodes: [1],
    });
  });
});
