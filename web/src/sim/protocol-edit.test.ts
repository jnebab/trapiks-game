import { describe, expect, it } from 'vitest';
import { deltaShape } from './delta-arrays';
import { isMainMessage, isWorkerMessage } from './protocol';

function deltaArrays(): Record<string, unknown> {
  const columns = Object.fromEntries(
    Object.entries(deltaShape).map(([name, ctor]) => [name, new ctor(0)]),
  );
  return { roadCount: 3, nodeCount: 4, ...columns };
}

describe('edit messages', () => {
  it('accepts command and quote messages from the main thread', () => {
    const command = { DeleteRoad: { road: 3 } };
    expect(isMainMessage({ type: 'command', command })).toBe(true);
    expect(isMainMessage({ type: 'command', command: 'Undo' })).toBe(true);
    const roundabout = { BuildRoundabout: { node: 2, radius_m: 18 } };
    expect(isMainMessage({ type: 'command', command: roundabout })).toBe(true);
    expect(isMainMessage({ type: 'quote', id: 1, command })).toBe(true);
    expect(isMainMessage({ type: 'command', command: { Explode: {} } })).toBe(false);
    expect(isMainMessage({ type: 'command', command: 'Redo' })).toBe(false);
    expect(isMainMessage({ type: 'quote', command })).toBe(false);
  });

  it('accepts quote results', () => {
    expect(isWorkerMessage({ type: 'quoteResult', session: 0, id: 1, result: { Ok: 120 } })).toBe(
      true,
    );
    expect(
      isWorkerMessage({ type: 'quoteResult', session: 0, id: 1, result: { Err: 'NoChange' } }),
    ).toBe(true);
    expect(isWorkerMessage({ type: 'quoteResult', session: 0, id: 1, result: { Ok: '1' } })).toBe(
      false,
    );
    expect(isWorkerMessage({ type: 'quoteResult', session: 0, result: { Ok: 1 } })).toBe(false);
  });
});

describe('command results messages', () => {
  const okResult = { seq: 0, outcome: { Ok: { cost: 5, changed_roads: [1], changed_nodes: [] } } };
  const errResult = { seq: 1, outcome: { Err: 'NothingToUndo' } };
  const budget = { limit: null, spent: 5 };

  it('accepts command results with or without a delta', () => {
    const results = [okResult, errResult];
    const base = { type: 'commandResults', session: 0, results, budget, log: [] };
    expect(isWorkerMessage({ ...base, delta: deltaArrays() })).toBe(true);
    expect(isWorkerMessage({ ...base, delta: null })).toBe(true);
    expect(isWorkerMessage({ ...base, budget: { limit: 10, spent: 0 }, delta: null })).toBe(true);
  });

  it('rejects malformed command results', () => {
    const base = {
      type: 'commandResults',
      session: 0,
      results: [okResult],
      budget,
      delta: null,
      log: [],
    };
    expect(isWorkerMessage({ ...base, delta: { ...deltaArrays(), roadIds: [1] } })).toBe(false);
    expect(isWorkerMessage({ ...base, delta: { ...deltaArrays(), roadCount: '3' } })).toBe(false);
    expect(isWorkerMessage({ ...base, budget: { spent: 1 } })).toBe(false);
    expect(isWorkerMessage({ ...base, results: [{ seq: 0, outcome: { Ok: {} } }] })).toBe(false);
    expect(isWorkerMessage({ ...base, delta: undefined })).toBe(false);
  });
});
