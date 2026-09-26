import type { NodeInspection } from '../../generated/NodeInspection';
import type { RoadInspection } from '../../generated/RoadInspection';
import { el } from '../dom';
import type { ViewContext } from './controls';
import { buildNodeView } from './node-view';
import { buildRoadView } from './road-view';

export interface InspectorPanel {
  element: HTMLElement;
  showRoad: (info: RoadInspection) => void;
  showNode: (info: NodeInspection) => void;
  hide: () => void;
}

export function createInspectorPanel(ctx: ViewContext): InspectorPanel {
  const element = el('aside', 'inspector');
  element.id = 'inspector';
  element.setAttribute('role', 'region');
  element.setAttribute('aria-label', 'Inspector');
  element.hidden = true;
  const show = (body: HTMLElement): void => {
    ctx.tooltip.hide();
    element.replaceChildren(body);
    element.hidden = false;
  };
  return {
    element,
    showRoad: (info) => {
      show(buildRoadView(info, ctx));
    },
    showNode: (info) => {
      show(buildNodeView(info, ctx));
    },
    hide: () => {
      ctx.tooltip.hide();
      element.replaceChildren();
      element.hidden = true;
    },
  };
}
