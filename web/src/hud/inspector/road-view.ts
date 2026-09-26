import type { RoadInspection } from '../../generated/RoadInspection';
import { el } from '../dom';
import { humanize, roadTitle } from './arm-labels';
import { actionButton } from './action-button';
import {
  commandButton,
  header,
  pressed,
  row,
  segmented,
  stepper,
  type ViewContext,
} from './controls';
import { directionCommands, laneStep, speedStep, type LaneDirection } from './view-models';

const SPEED_STEP = 10;

function laneStepper(info: RoadInspection, dir: LaneDirection, ctx: ViewContext): HTMLElement {
  const label = dir === 'forward' ? 'Lanes →' : 'Lanes ←';
  const value = dir === 'forward' ? info.lanes_forward : info.lanes_backward;
  return stepper(
    label,
    String(value),
    [
      { text: '−', action: 'Remove lane', command: laneStep(info, dir, -1) },
      { text: '+', action: 'Add lane', command: laneStep(info, dir, 1) },
    ],
    ctx,
  );
}

function directionRow(info: RoadInspection, ctx: ViewContext): HTMLElement {
  const buttons = directionCommands(info).map((option) => {
    const button = commandButton(
      { text: option.label, action: option.label, command: option.command },
      ctx,
    );
    return pressed(button, option.current);
  });
  return row('Direction', segmented(buttons));
}

function speedStepper(info: RoadInspection, ctx: ViewContext): HTMLElement {
  return stepper(
    'Speed',
    `${String(info.speed_kph)} km/h`,
    [
      { text: '−', action: 'Lower speed limit', command: speedStep(info, -SPEED_STEP) },
      { text: '+', action: 'Raise speed limit', command: speedStep(info, SPEED_STEP) },
    ],
    ctx,
  );
}

function deleteButton(info: RoadInspection, ctx: ViewContext): HTMLElement {
  const button = actionButton(
    'Delete road',
    () => ({ DeleteRoad: { road: info.road } }),
    ctx.client,
    ctx.tooltip,
  );
  button.classList.add('danger');
  return button;
}

function subtitle(info: RoadInspection): string {
  return `${humanize(info.class)} · ${String(Math.round(info.length_m))} m`;
}

export function buildRoadView(info: RoadInspection, ctx: ViewContext): HTMLElement {
  const root = el('div', 'inspector-body');
  root.dataset.lanesForward = String(info.lanes_forward);
  root.dataset.lanesBackward = String(info.lanes_backward);
  root.dataset.speed = String(info.speed_kph);
  root.append(header(roadTitle(info.road, ctx.stores), subtitle(info)));
  if (info.deleted) {
    root.append(el('p', 'inspector-note', 'This road has been removed.'));
    return root;
  }
  root.append(
    laneStepper(info, 'forward', ctx),
    laneStepper(info, 'backward', ctx),
    directionRow(info, ctx),
    speedStepper(info, ctx),
    deleteButton(info, ctx),
  );
  return root;
}
