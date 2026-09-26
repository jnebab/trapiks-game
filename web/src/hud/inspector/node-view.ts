import type { Control } from '../../generated/Control';
import type { NodeInspection } from '../../generated/NodeInspection';
import type { TurnKind } from '../../generated/TurnKind';
import { el } from '../dom';
import { armLabels, roadTitle } from './arm-labels';
import { actionButton } from './action-button';
import {
  commandButton,
  header,
  pressed,
  row,
  section,
  segmented,
  type ViewContext,
} from './controls';
import { signalTiming } from './signal-timing';
import {
  flyoverLabel,
  roundaboutActions,
  turnGroups,
  type RoundaboutAction,
  type TurnGroup,
} from './view-models';

const CONTROLS: [Control, string][] = [
  ['Priority', 'Priority'],
  ['Yield', 'Yield'],
  ['Stop', 'Stop'],
  ['AllWayStop', 'All-way stop'],
  ['Signal', 'Signal'],
];

const ARROWS: Record<TurnKind, string> = {
  Right: '↱',
  Through: '↑',
  Left: '↰',
  UTurn: '↶',
};

function roadNames(info: NodeInspection, ctx: ViewContext): string {
  const names = new Set(info.roads.map((road) => roadTitle(road, ctx.stores)));
  return [...names].join(' · ');
}

function controlRow(info: NodeInspection, ctx: ViewContext): HTMLElement {
  const buttons = CONTROLS.map(([control, label]) => {
    const command = { SetJunctionControl: { node: info.node, control } };
    return pressed(
      commandButton({ text: label, action: label, command }, ctx),
      control === info.control,
    );
  });
  return row('Control', segmented(buttons));
}

function turnGroupView(node: number, group: TurnGroup, ctx: ViewContext): HTMLElement {
  const toggles = group.turns.map(({ turn, label }) => {
    const text = `${ARROWS[turn.kind]} ${label}`;
    const command = {
      SetTurnAllowed: {
        node,
        from_road: turn.from_road,
        to_road: turn.to_road,
        allowed: !turn.allowed,
      },
    };
    const action = turn.allowed ? 'Ban turn' : 'Allow turn';
    const button = pressed(commandButton({ text, action, command }, ctx), turn.allowed);
    button.classList.add('turn-toggle');
    button.classList.toggle('banned', !turn.allowed);
    return button;
  });
  const element = el('div', 'turn-group');
  element.append(el('span', 'inspector-label', `From ${group.title}`), segmented(toggles));
  return element;
}

function turnsSection(
  info: NodeInspection,
  labels: Map<number, string>,
  ctx: ViewContext,
): HTMLElement {
  const groups = turnGroups(info, labels).map((group) => turnGroupView(info.node, group, ctx));
  return section('Turns', ...groups);
}

function flyoverSection(
  info: NodeInspection,
  labels: Map<number, string>,
  ctx: ViewContext,
): HTMLElement {
  const buttons = info.flyover_pairs.map((pair) => {
    const label = flyoverLabel(pair, labels);
    const through: [number, number] = [pair[0], pair[1]];
    return actionButton(
      label,
      () => ({ BuildFlyover: { node: info.node, through } }),
      ctx.client,
      ctx.tooltip,
    );
  });
  return section('Flyover', ...buttons);
}

function roundaboutRow(actions: RoundaboutAction[], ctx: ViewContext): HTMLElement {
  const buttons = actions.map(({ text, action, command }) =>
    actionButton({ text, action }, () => command, ctx.client, ctx.tooltip),
  );
  return row('Roundabout', segmented(buttons));
}

export function buildNodeView(info: NodeInspection, ctx: ViewContext): HTMLElement {
  const root = el('div', 'inspector-body');
  root.dataset.control = info.control;
  root.dataset.turnsBanned = String(info.turns.filter((turn) => !turn.allowed).length);
  const labels = armLabels(info.roads, info.node, ctx.stores);
  root.append(header('Junction', roadNames(info, ctx)), controlRow(info, ctx));
  if (info.signal !== null) {
    root.append(signalTiming(info.node, info.signal, ctx));
  }
  if (info.turns.length > 0) {
    root.append(turnsSection(info, labels, ctx));
  }
  if (info.flyover_pairs.length > 0) {
    root.append(flyoverSection(info, labels, ctx));
  }
  const roundabouts = roundaboutActions(info);
  if (roundabouts.length > 0) {
    root.append(roundaboutRow(roundabouts, ctx));
  }
  return root;
}
