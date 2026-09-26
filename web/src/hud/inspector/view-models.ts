import type { EditCommand } from '../../generated/EditCommand';
import type { NodeInspection } from '../../generated/NodeInspection';
import type { QuoteOutcome } from '../../generated/QuoteOutcome';
import type { RoadInspection } from '../../generated/RoadInspection';
import type { TurnInfo } from '../../generated/TurnInfo';
import { errorMessage } from '../messages';
import { formatPesos } from '../money';

const MAX_LANES = 8;
const MIN_KPH = 10;
const MAX_KPH = 120;

export type LaneDirection = 'forward' | 'backward';

export interface DirectionOption {
  label: string;
  command: EditCommand;
  current: boolean;
}

export interface TurnGroup {
  from: number;
  title: string;
  turns: { turn: TurnInfo; label: string }[];
}

function setLanes(road: RoadInspection, forward: number, backward: number): EditCommand {
  return { SetLanes: { road: road.road, forward, backward } };
}

export function directionCommands(road: RoadInspection): DirectionOption[] {
  const { lanes_forward: f, lanes_backward: b } = road;
  const total = f + b;
  const twoWay: [number, number] = [
    Math.max(1, Math.ceil(total / 2)),
    Math.max(1, Math.floor(total / 2)),
  ];
  const options: [string, [number, number], boolean][] = [
    ['Two-way', f > 0 && b > 0 ? [f, b] : twoWay, f > 0 && b > 0],
    ['One-way →', b === 0 ? [f, b] : [Math.min(MAX_LANES, total), 0], b === 0],
    ['One-way ←', f === 0 ? [f, b] : [0, Math.min(MAX_LANES, total)], f === 0],
  ];
  return options.map(([label, [forward, backward], current]) => ({
    label,
    command: setLanes(road, forward, backward),
    current,
  }));
}

export function laneStep(
  road: RoadInspection,
  dir: LaneDirection,
  delta: number,
): EditCommand | null {
  const forward = road.lanes_forward + (dir === 'forward' ? delta : 0);
  const backward = road.lanes_backward + (dir === 'backward' ? delta : 0);
  const inRange = [forward, backward].every((lanes) => lanes >= 0 && lanes <= MAX_LANES);
  if (!inRange || forward + backward === 0) {
    return null;
  }
  return setLanes(road, forward, backward);
}

export function speedStep(road: RoadInspection, delta: number): EditCommand | null {
  const kph = road.speed_kph + delta;
  if (kph < MIN_KPH || kph > MAX_KPH) {
    return null;
  }
  return { SetSpeedLimit: { road: road.road, kph } };
}

function labelOf(labels: ReadonlyMap<number, string>, road: number): string {
  return labels.get(road) ?? `Road #${String(road)}`;
}

export function turnGroups(info: NodeInspection, labels: ReadonlyMap<number, string>): TurnGroup[] {
  const groups = new Map<number, TurnGroup>();
  for (const turn of info.turns) {
    const group = groups.get(turn.from_road) ?? {
      from: turn.from_road,
      title: labelOf(labels, turn.from_road),
      turns: [],
    };
    group.turns.push({ turn, label: labelOf(labels, turn.to_road) });
    groups.set(turn.from_road, group);
  }
  return [...groups.values()].sort((a, b) => a.from - b.from);
}

export function flyoverLabel(
  pair: readonly [number, number],
  labels: ReadonlyMap<number, string>,
): string {
  return `Build flyover: ${labelOf(labels, pair[0])} ↔ ${labelOf(labels, pair[1])}`;
}

export function costLabel(outcome: QuoteOutcome, label: string): string {
  if ('Err' in outcome) {
    return errorMessage(outcome.Err);
  }
  return `${formatPesos(outcome.Ok)} · ${label}`;
}
