import { Graphics, type Container } from 'pixi.js';
import type { RoadStore } from './road-store';
import { laneWidth } from './tiles/road-style';
import { tracePolyline } from './tiles/trace';

const HIGHLIGHT = 0x8ed8f6;
const HOVER_ALPHA = 0.55;
const SELECTED_ALPHA = 0.8;
const HANDLE_COLOR = 0x151515;
const HANDLE_RADIUS = 0.6;
const HANDLE_MIN_PX = 3;

export class SelectionLayer {
  hovered: number | undefined;
  selected: number | undefined;
  private readonly hoverGraphics = new Graphics();
  private readonly selectGraphics = new Graphics();
  private handleRadius = HANDLE_RADIUS;

  constructor(
    container: Container,
    private readonly roads: RoadStore,
  ) {
    container.addChild(this.hoverGraphics, this.selectGraphics);
  }

  setHover(road: number | undefined): void {
    if (road !== this.hovered) {
      this.hovered = road;
      this.drawHover();
    }
  }

  setSelected(road: number | undefined): void {
    if (road !== this.selected) {
      this.selected = road;
      this.drawSelected();
    }
  }

  setScale(scale: number): void {
    const radius = Math.max(HANDLE_RADIUS, HANDLE_MIN_PX / scale);
    if (radius !== this.handleRadius) {
      this.handleRadius = radius;
      this.drawSelected();
    }
  }

  refresh(changed: ReadonlySet<number>): void {
    if (this.hovered !== undefined && changed.has(this.hovered)) {
      this.drawHover();
    }
    if (this.selected !== undefined && changed.has(this.selected)) {
      this.drawSelected();
    }
  }

  private drawHover(): void {
    this.hoverGraphics.clear();
    if (this.hovered !== undefined) {
      this.drawRoad(this.hoverGraphics, this.hovered, HOVER_ALPHA);
    }
  }

  private drawSelected(): void {
    const g = this.selectGraphics;
    g.clear();
    if (this.selected === undefined) {
      return;
    }
    this.drawRoad(g, this.selected, SELECTED_ALPHA);
    const points = this.roads.pointsOf(this.selected);
    for (const p of [points[0], points[points.length - 1]]) {
      if (p !== undefined) {
        g.circle(p.x, p.y, this.handleRadius).fill(HANDLE_COLOR);
      }
    }
  }

  private drawRoad(g: Graphics, road: number, alpha: number): void {
    if (this.roads.isDeleted(road)) {
      return;
    }
    tracePolyline(g, this.roads, road);
    g.stroke({
      width: laneWidth(this.roads, road),
      color: HIGHLIGHT,
      alpha,
      join: 'round',
      cap: 'round',
    });
  }
}
