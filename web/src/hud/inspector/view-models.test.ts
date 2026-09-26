import { describe, expect, it } from 'vitest';
import type { NodeInspection } from '../../generated/NodeInspection';
import type { RoadInspection } from '../../generated/RoadInspection';
import { armLabel, armLabels, type ArmStores } from './arm-labels';
import {
  costLabel,
  directionCommands,
  flyoverLabel,
  laneStep,
  roundaboutActions,
  speedStep,
  turnGroups,
} from './view-models';

function road(lanes_forward: number, lanes_backward: number, speed_kph = 50): RoadInspection {
  return {
    road: 4,
    class: 'Primary',
    length_m: 120,
    lanes_forward,
    lanes_backward,
    speed_kph,
    layer: 0,
    deleted: false,
  };
}

function lanesOf(road: RoadInspection): [number, number][] {
  return directionCommands(road).map(({ command }) => {
    if (typeof command === 'string' || !('SetLanes' in command)) {
      throw new Error('expected SetLanes');
    }
    return [command.SetLanes.forward, command.SetLanes.backward];
  });
}

function current(road: RoadInspection): string[] {
  return directionCommands(road)
    .filter((option) => option.current)
    .map((option) => option.label);
}

describe('directionCommands', () => {
  it('keeps the lane total for each option and marks the current one', () => {
    expect(lanesOf(road(2, 2))).toEqual([
      [2, 2],
      [4, 0],
      [0, 4],
    ]);
    expect(current(road(2, 2))).toEqual(['Two-way']);
    expect(lanesOf(road(3, 0))).toEqual([
      [2, 1],
      [3, 0],
      [0, 3],
    ]);
    expect(current(road(3, 0))).toEqual(['One-way →']);
    expect(lanesOf(road(0, 1))).toEqual([
      [1, 1],
      [1, 0],
      [0, 1],
    ]);
    expect(current(road(0, 1))).toEqual(['One-way ←']);
    expect(lanesOf(road(3, 1))).toEqual([
      [3, 1],
      [4, 0],
      [0, 4],
    ]);
    expect(current(road(3, 1))).toEqual(['Two-way']);
  });

  it('caps one-way conversions at eight lanes', () => {
    expect(lanesOf(road(5, 5))).toEqual([
      [5, 5],
      [8, 0],
      [0, 8],
    ]);
  });
});

describe('steps', () => {
  it('bounds lane steps to 0..8 per direction and at least one lane', () => {
    expect(laneStep(road(1, 1), 'forward', 1)).toEqual({
      SetLanes: { road: 4, forward: 2, backward: 1 },
    });
    expect(laneStep(road(8, 1), 'forward', 1)).toBeNull();
    expect(laneStep(road(0, 1), 'forward', -1)).toBeNull();
    expect(laneStep(road(0, 1), 'backward', -1)).toBeNull();
    expect(laneStep(road(1, 1), 'backward', -1)).toEqual({
      SetLanes: { road: 4, forward: 1, backward: 0 },
    });
  });

  it('bounds speed steps to 10..120', () => {
    expect(speedStep(road(1, 1, 50), 10)).toEqual({ SetSpeedLimit: { road: 4, kph: 60 } });
    expect(speedStep(road(1, 1, 120), 10)).toBeNull();
    expect(speedStep(road(1, 1, 10), -10)).toBeNull();
    expect(speedStep(road(1, 1, 20), -10)).toEqual({ SetSpeedLimit: { road: 4, kph: 10 } });
  });
});

const points = new Map([
  [
    0,
    [
      { x: 0, y: 0 },
      { x: 0, y: -100 },
    ],
  ],
  [
    1,
    [
      { x: 0, y: 100 },
      { x: 0, y: 0 },
    ],
  ],
  [
    2,
    [
      { x: 0, y: 0 },
      { x: 80, y: 5 },
    ],
  ],
]);

const stores: ArmStores = {
  roads: {
    from: (id) => (id === 1 ? 9 : 0),
    name: (id) => (id === 2 ? 2 : 1),
    classCode: () => 0,
    pointsOf: (id) => points.get(id) ?? [],
  },
  names: ['', 'Avenue 5', 'Road 3'],
  classNames: ['Primary'],
};

describe('arm labels', () => {
  it('keeps two same-named arms distinct by compass letter', () => {
    expect(armLabel(0, 0, stores)).toBe('Avenue 5 N');
    expect(armLabel(1, 0, stores)).toBe('Avenue 5 S');
    expect(armLabel(2, 0, stores)).toBe('Road 3 E');
  });

  it('groups turns by from road with arm labels', () => {
    const info: NodeInspection = {
      node: 0,
      control: 'Priority',
      roads: [0, 1, 2],
      signal: null,
      turns: [
        { from_road: 1, to_road: 0, kind: 'Through', allowed: true },
        { from_road: 0, to_road: 1, kind: 'Through', allowed: true },
        { from_road: 0, to_road: 2, kind: 'Left', allowed: false },
      ],
      flyover_pairs: [[0, 1]],
    };
    const labels = armLabels(info.roads, info.node, stores);
    const groups = turnGroups(info, labels);
    expect(groups.map((group) => [group.from, group.title])).toEqual([
      [0, 'Avenue 5 N'],
      [1, 'Avenue 5 S'],
    ]);
    expect(groups[0]?.turns.map((turn) => turn.label)).toEqual(['Avenue 5 S', 'Road 3 E']);
    expect(flyoverLabel([0, 1], labels)).toBe('Build flyover: Avenue 5 N ↔ Avenue 5 S');
  });
});

describe('roundaboutActions', () => {
  it('offers small and large roundabouts at a junction', () => {
    const info: NodeInspection = {
      node: 0,
      control: 'Priority',
      roads: [0, 1, 2],
      signal: null,
      turns: [],
      flyover_pairs: [],
    };
    expect(roundaboutActions(info)).toEqual([
      {
        text: 'Small',
        action: 'Small roundabout',
        command: { BuildRoundabout: { node: 0, radius_m: 18 } },
      },
      {
        text: 'Large',
        action: 'Large roundabout',
        command: { BuildRoundabout: { node: 0, radius_m: 28 } },
      },
    ]);
    expect(roundaboutActions({ ...info, roads: [0, 1] })).toEqual([]);
    expect(costLabel({ Ok: 2104 }, 'Small roundabout')).toBe('₱2,104 · Small roundabout');
  });
});

describe('costLabel', () => {
  it('formats a quote in pesos or an error message', () => {
    expect(costLabel({ Ok: 60 }, 'Add lane')).toBe('₱60 · Add lane');
    expect(costLabel({ Ok: 2500 }, 'Build flyover')).toBe('₱2,500 · Build flyover');
    expect(costLabel({ Err: 'InsufficientBudget' }, 'Add lane')).toBe('Not enough budget');
  });
});
