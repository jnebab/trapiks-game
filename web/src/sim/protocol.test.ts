import { describe, expect, it } from 'vitest';
import { isWorkerMessage } from './protocol';

function readyMessage(): Record<string, unknown> {
  return {
    type: 'ready',
    session: 0,
    mode: 'sandbox',
    region: null,
    meta: {
      map_hash: '0123456789abcdef',
      road_count: 1,
      node_count: 2,
      area_ring_count: 0,
      class_names: ['Motorway'],
      class_ranks: [14],
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
      roundabout: new Uint8Array(1),
      from: new Uint32Array(1),
      to: new Uint32Array(1),
    },
    nodes: { x: new Float32Array(2), y: new Float32Array(2), controlCode: new Uint8Array(2) },
    areas: emptyAreas(),
    ...streetArrays(),
  };
}

function emptyAreas(): Record<string, unknown> {
  return {
    kindCode: new Uint8Array(0),
    hole: new Uint8Array(0),
    ringStart: new Uint32Array([0]),
    x: new Float32Array(0),
    y: new Float32Array(0),
  };
}

function streetArrays(): Record<string, unknown> {
  return {
    roadSetbacks: new Float32Array(2),
    junctions: {
      node: new Uint32Array(0),
      layer: new Int8Array(0),
      minLayer: new Int8Array(0),
      filletLayer: new Int8Array(0),
      ringStart: new Uint32Array([0]),
      x: new Float32Array(0),
      y: new Float32Array(0),
    },
    markers: {
      link: new Uint32Array(0),
      node: new Uint32Array(0),
      kind: new Uint8Array(0),
      x1: new Float32Array(0),
      y1: new Float32Array(0),
      x2: new Float32Array(0),
      y2: new Float32Array(0),
    },
    signalPills: {
      link: new Uint32Array(0),
      x: new Float32Array(0),
      y: new Float32Array(0),
      angle: new Float32Array(0),
    },
  };
}

describe('isWorkerMessage', () => {
  it('accepts a valid ready message', () => {
    expect(isWorkerMessage(readyMessage())).toBe(true);
  });

  it('accepts an error message', () => {
    expect(isWorkerMessage({ type: 'error', session: 0, message: 'boom' })).toBe(true);
  });

  it('rejects a ready message with a wrong array type', () => {
    const message = readyMessage();
    message.roads = { ...(message.roads as object), layer: new Uint8Array(1) };
    expect(isWorkerMessage(message)).toBe(false);
  });

  it('rejects a ready message without junction shapes', () => {
    const message = readyMessage();
    delete message.junctions;
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
    expect(isWorkerMessage({ type: 'error', session: 0 })).toBe(false);
  });
});

function buffers(): Record<string, unknown> {
  return {
    ids: new Uint32Array(4),
    x: new Float32Array(4),
    y: new Float32Array(4),
    heading: new Float32Array(4),
    style: new Uint8Array(4),
  };
}

function stats(): Record<string, unknown> {
  return {
    tick: 10,
    sim_time_s: 1,
    active: 2,
    created: 3,
    spawned: 3,
    arrivals: 1,
    unserved: 0,
    stranded: 0,
    mean_travel_s: 0,
    mean_delay_s: 0,
    accrued_delay_s: 0,
    throughput_per_hour: 0,
    mean_speed: 4,
    stopped_share: 0,
  };
}

describe('snapshot and stats messages', () => {
  it('accepts a valid snapshot', () => {
    const message = {
      type: 'snapshot',
      session: 0,
      tick: 3,
      simTime: 0.3,
      count: 2,
      buffers: buffers(),
    };
    expect(isWorkerMessage(message)).toBe(true);
  });

  it('rejects a snapshot with a wrong buffer type', () => {
    const wrong = { ...buffers(), style: new Float32Array(4) };
    const message = {
      type: 'snapshot',
      session: 0,
      tick: 3,
      simTime: 0.3,
      count: 2,
      buffers: wrong,
    };
    expect(isWorkerMessage(message)).toBe(false);
    expect(
      isWorkerMessage({ type: 'snapshot', session: 0, tick: 3, simTime: 0.3, buffers: buffers() }),
    ).toBe(false);
  });

  it('accepts stats and rejects stats without the ratio or step arrays', () => {
    const ratio = new Float32Array(3);
    const stepMs = new Float64Array(2);
    const message = { type: 'stats', session: 0, stats: stats(), roadSpeedRatio: ratio, stepMs };
    expect(isWorkerMessage(message)).toBe(true);
    expect(isWorkerMessage({ ...message, roadSpeedRatio: [1] })).toBe(false);
    expect(isWorkerMessage({ ...message, stats: { tick: 1 } })).toBe(false);
    expect(isWorkerMessage({ ...message, stepMs: [1] })).toBe(false);
  });
});

describe('signals message', () => {
  it('accepts a byte array of states and rejects anything else', () => {
    expect(
      isWorkerMessage({ type: 'signals', session: 0, states: new Uint8Array([0, 1, 2]) }),
    ).toBe(true);
    expect(isWorkerMessage({ type: 'signals', session: 0, states: [0, 1, 2] })).toBe(false);
    expect(isWorkerMessage({ type: 'signals', session: 0 })).toBe(false);
  });
});
