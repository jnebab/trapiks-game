import { Graphics, type Container } from 'pixi.js';
import type { InspectTarget } from '../sim/protocol';
import { nodeConnections, roadConnections, type Connections } from './connections';
import { isLevelShown, type LevelView } from './level-view';
import type { NodeStore } from './node-store';
import type { RoadStore } from './road-store';
import { laneWidth } from './tiles/road-style';
import { tracePolyline } from './tiles/trace';

const OUTGOING = 0x35b36b;
const INCOMING = 0xf29a38;
const DOT_COLOR = 0x1f6f8b;
const ALPHA = 0.6;
const WIDTH_EXTRA = 1.5;
const INNER_FACTOR = 0.45;
const DOT_RADIUS = 1.4;
const DOT_MIN_PX = 4;

const EMPTY: Connections = { outgoing: [], incoming: [], nodes: [] };

export interface ConnectionSources {
  roads: RoadStore;
  nodes: NodeStore;
  levelView: () => LevelView;
}

export class ConnectionHighlight {
  private readonly graphics = new Graphics();
  private hovered: number | undefined;
  private selected: InspectTarget | undefined;
  private shown: Connections = EMPTY;
  private dotRadius = DOT_RADIUS;

  constructor(
    container: Container,
    private readonly sources: ConnectionSources,
  ) {
    container.addChild(this.graphics);
  }

  setHover(road: number | undefined): void {
    if (road !== this.hovered) {
      this.hovered = road;
      this.draw();
    }
  }

  setSelected(target: InspectTarget | undefined): void {
    this.selected = target;
    this.draw();
  }

  setScale(scale: number): void {
    const radius = Math.max(DOT_RADIUS, DOT_MIN_PX / scale);
    if (radius !== this.dotRadius) {
      this.dotRadius = radius;
      this.draw();
    }
  }

  summary(): string {
    const { outgoing, incoming, nodes } = this.shown;
    return `${String(outgoing.length)}/${String(incoming.length)}/${String(nodes.length)}`;
  }

  draw(): void {
    this.graphics.clear();
    this.shown = this.visible(this.connections());
    const incoming = new Set(this.shown.incoming);
    for (const road of this.shown.incoming) {
      this.strokeRoad(road, INCOMING, 1);
    }
    for (const road of this.shown.outgoing) {
      this.strokeRoad(road, OUTGOING, incoming.has(road) ? INNER_FACTOR : 1);
    }
    for (const node of this.shown.nodes) {
      const { nodes } = this.sources;
      this.graphics.circle(nodes.x(node), nodes.y(node), this.dotRadius).fill(DOT_COLOR);
    }
  }

  private connections(): Connections {
    const { roads } = this.sources;
    if (this.hovered !== undefined) {
      return roadConnections(roads, this.hovered);
    }
    if (this.selected === undefined) {
      return EMPTY;
    }
    if ('road' in this.selected) {
      return roadConnections(roads, this.selected.road);
    }
    return nodeConnections(roads, this.selected.node);
  }

  private visible(found: Connections): Connections {
    const view = this.sources.levelView();
    const shown = (road: number): boolean => isLevelShown(view, this.sources.roads.layer(road));
    return {
      ...found,
      outgoing: found.outgoing.filter(shown),
      incoming: found.incoming.filter(shown),
    };
  }

  private strokeRoad(road: number, color: number, factor: number): void {
    const { roads } = this.sources;
    const width = (laneWidth(roads, road) + WIDTH_EXTRA) * factor;
    tracePolyline(this.graphics, roads, road);
    this.graphics.stroke({ width, color, alpha: ALPHA, join: 'round', cap: 'round' });
  }
}
