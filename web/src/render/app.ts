import { Application, Container } from 'pixi.js';
import { palette } from './palette';

export interface DebugApp {
  app: Application;
  world: Container;
}

export async function createApp(root: HTMLElement): Promise<DebugApp> {
  const app = new Application();
  await app.init({ resizeTo: window, background: palette.ground, antialias: true });
  root.appendChild(app.canvas);
  const world = new Container();
  app.stage.addChild(world);
  return { app, world };
}
