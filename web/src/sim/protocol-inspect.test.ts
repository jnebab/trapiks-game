import { describe, expect, it } from 'vitest';
import { isMainMessage, isWorkerMessage } from './protocol';

const road = {
  road: 3,
  class: 'Residential',
  length_m: 150,
  lanes_forward: 1,
  lanes_backward: 1,
  speed_kph: 20,
  layer: 0,
  deleted: false,
};

const node = {
  node: 7,
  control: 'Signal',
  roads: [1, 2, 3],
  signal: { phase_count: 2, greens_s: [30, 30], offset_s: 0 },
  turns: [{ from_road: 1, to_road: 2, kind: 'Through', allowed: true }],
  flyover_pairs: [[1, 2]],
};

describe('inspect messages', () => {
  it('accepts inspect requests for a road or a node', () => {
    expect(isMainMessage({ type: 'inspect', id: 1, target: { road: 3 } })).toBe(true);
    expect(isMainMessage({ type: 'inspect', id: 1, target: { node: 3 } })).toBe(true);
    expect(isMainMessage({ type: 'inspect', id: 1, target: { road: 3, node: 1 } })).toBe(false);
    expect(isMainMessage({ type: 'inspect', id: 1, target: { lane: 3 } })).toBe(false);
    expect(isMainMessage({ type: 'inspect', target: { road: 3 } })).toBe(false);
  });

  it('accepts road, node and empty inspections', () => {
    expect(isWorkerMessage({ type: 'inspection', session: 0, id: 1, target: { road } })).toBe(true);
    expect(isWorkerMessage({ type: 'inspection', session: 0, id: 1, target: { node } })).toBe(true);
    const unsignalized = { ...node, signal: null };
    expect(
      isWorkerMessage({ type: 'inspection', session: 0, id: 1, target: { node: unsignalized } }),
    ).toBe(true);
    expect(isWorkerMessage({ type: 'inspection', session: 0, id: 1, target: null })).toBe(true);
  });

  it('rejects malformed inspections', () => {
    const badRoad = { ...road, deleted: 0 };
    expect(
      isWorkerMessage({ type: 'inspection', session: 0, id: 1, target: { road: badRoad } }),
    ).toBe(false);
    const badTurns = { ...node, turns: [{ from_road: 1 }] };
    expect(
      isWorkerMessage({ type: 'inspection', session: 0, id: 1, target: { node: badTurns } }),
    ).toBe(false);
    const badPairs = { ...node, flyover_pairs: [[1]] };
    expect(
      isWorkerMessage({ type: 'inspection', session: 0, id: 1, target: { node: badPairs } }),
    ).toBe(false);
    expect(isWorkerMessage({ type: 'inspection', session: 0, target: null })).toBe(false);
  });
});
