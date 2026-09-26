import { Graphics, type Container } from 'pixi.js';
import type { Point } from '../../render/polyline';

const OK_COLOR = 0xffffff;
const OK_ALPHA = 0.6;
const ERROR_COLOR = 0xe5484d;
const ERROR_ALPHA = 0.5;
const MISS_WIDTH_PX = 2;
const MARKER_RADIUS_PX = 4;
const MARKER_COLOR = 0x151515;

function trace(g: Graphics, points: readonly Point[]): void {
  points.forEach((p, i) => {
    if (i === 0) {
      g.moveTo(p.x, p.y);
    } else {
      g.lineTo(p.x, p.y);
    }
  });
}

export class PreviewLayer {
  private readonly graphics = new Graphics();
  private scale = 1;

  constructor(container: Container) {
    container.addChild(this.graphics);
  }

  setScale(scale: number): void {
    this.scale = scale;
  }

  road(points: readonly Point[], width: number, ok: boolean): void {
    const g = this.graphics.clear();
    trace(g, points);
    g.stroke({
      width,
      color: ok ? OK_COLOR : ERROR_COLOR,
      alpha: ok ? OK_ALPHA : ERROR_ALPHA,
      join: 'round',
      cap: 'round',
    });
    this.markers(points);
  }

  miss(from: Point, to: Point): void {
    const g = this.graphics.clear();
    trace(g, [from, to]);
    g.stroke({ width: MISS_WIDTH_PX / this.scale, color: ERROR_COLOR, alpha: 1 });
    this.markers([from]);
  }

  marker(point: Point): void {
    this.graphics.clear();
    this.markers([point]);
  }

  clear(): void {
    this.graphics.clear();
  }

  destroy(): void {
    this.graphics.destroy();
  }

  private markers(points: readonly Point[]): void {
    const radius = MARKER_RADIUS_PX / this.scale;
    for (const p of [points[0], points[points.length - 1]]) {
      if (p !== undefined) {
        this.graphics.circle(p.x, p.y, radius).fill(MARKER_COLOR);
      }
    }
  }
}
