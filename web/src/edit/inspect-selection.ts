import type { CommandResult } from '../generated/CommandResult';
import type { InspectorPanel } from '../hud/inspector/panel';
import type { ConnectionHighlight } from '../render/connection-highlight';
import type { DetailStore } from '../render/detail-store';
import type { NodeHighlight } from '../render/node-highlight';
import type { SelectionLayer } from '../render/selection';
import type { SimClient } from '../sim/client';
import type { InspectTarget, Inspection } from '../sim/protocol';

export interface InspectDeps {
  client: Pick<SimClient, 'inspect'>;
  panel: InspectorPanel;
  roads: SelectionLayer;
  node: NodeHighlight;
  connections: ConnectionHighlight;
  detail: DetailStore;
}

function touches(result: CommandResult, target: InspectTarget): boolean {
  if ('Err' in result.outcome) {
    return false;
  }
  const outcome = result.outcome.Ok;
  if ('road' in target) {
    return outcome.changed_roads.includes(target.road);
  }
  return outcome.changed_nodes.includes(target.node);
}

export class InspectSelection {
  target: InspectTarget | undefined;
  private token = 0;

  constructor(private readonly deps: InspectDeps) {}

  select(target: InspectTarget | undefined): void {
    this.target = target;
    this.deps.roads.setSelected(target !== undefined && 'road' in target ? target.road : undefined);
    this.deps.node.set(target !== undefined && 'node' in target ? target.node : undefined);
    this.deps.connections.setSelected(target);
    this.token += 1;
    if (target === undefined) {
      this.deps.panel.hide();
      return;
    }
    this.load(target);
  }

  onResults(results: readonly CommandResult[]): void {
    this.deps.node.draw();
    const target = this.target;
    if (target !== undefined && results.some((result) => touches(result, target))) {
      this.load(target);
    }
  }

  private load(target: InspectTarget): void {
    this.token += 1;
    const token = this.token;
    this.deps.client.inspect(target).then(
      (inspection) => {
        if (token === this.token) {
          this.show(inspection);
        }
      },
      () => undefined,
    );
  }

  private show(inspection: Inspection | null): void {
    if (inspection === null) {
      this.select(undefined);
      return;
    }
    if ('road' in inspection) {
      this.showRoad(inspection.road);
      return;
    }
    if (!this.deps.detail.junctions.has(inspection.node.node)) {
      this.select(undefined);
      return;
    }
    this.deps.panel.showNode(inspection.node);
  }

  private showRoad(road: Extract<Inspection, { road: unknown }>['road']): void {
    if (road.deleted) {
      this.select(undefined);
      return;
    }
    this.deps.panel.showRoad(road);
  }
}
