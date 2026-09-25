import { describe, expect, it } from 'vitest';
import { isWorkerMessage } from './protocol';

function readyMessage(): Record<string, unknown> {
  return {
    type: 'ready',
    meta: {
      map_hash: '0123456789abcdef',
      road_count: 1,
      node_count: 2,
      area_ring_count: 0,
      class_names: ['Motorway'],
      control_names: ['Priority'],
      area_kind_names: ['Water'],
      names: [],
      bounds: [0, 0, 1, 1],
    },
    roads: {
      pointStart: new Uint32Array([0, 2]),
      x: new Float32Array(2),
      y: new Float32Array(2),
      classCode: new Uint8Array(1),
      lanesForward: new Uint8Array(1),
      lanesBackward: new Uint8Array(1),
      layer: new Int8Array(1),
      name: new Uint32Array(1),
    },
    nodes: { x: new Float32Array(2), y: new Float32Array(2), controlCode: new Uint8Array(2) },
    areas: {
      kindCode: new Uint8Array(0),
      ringStart: new Uint32Array([0]),
      x: new Float32Array(0),
      y: new Float32Array(0),
    },
  };
}

describe('isWorkerMessage', () => {
  it('accepts a valid ready message', () => {
    expect(isWorkerMessage(readyMessage())).toBe(true);
  });

  it('accepts an error message', () => {
    expect(isWorkerMessage({ type: 'error', message: 'boom' })).toBe(true);
  });

  it('rejects a ready message with a wrong array type', () => {
    const message = readyMessage();
    message.roads = { ...(message.roads as object), layer: new Uint8Array(1) };
    expect(isWorkerMessage(message)).toBe(false);
  });

  it('rejects a ready message with missing meta fields', () => {
    const message = readyMessage();
    message.meta = { map_hash: 'x' };
    expect(isWorkerMessage(message)).toBe(false);
  });

  it('rejects unknown and non-object values', () => {
    expect(isWorkerMessage(null)).toBe(false);
    expect(isWorkerMessage({ type: 'nope' })).toBe(false);
    expect(isWorkerMessage({ type: 'error' })).toBe(false);
  });
});
