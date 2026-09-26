import { Engine, type MapHandle } from '../wasm/pkg/trapiks_sim_wasm.js';
import type { CommandResult } from '../generated/CommandResult';
import type { EditCommand } from '../generated/EditCommand';
import type { SimConfig } from '../generated/SimConfig';
import { isCommandResults } from '../sim/edit-values';
import { isMapMeta, type GameMode, type ReadyMessage, type Region } from '../sim/protocol';
import { EngineSession, type Post } from './engine-session';
import { collectGeometry, transferList } from './geometry';

export interface EngineRequest {
  session: number;
  config: SimConfig;
  mode: GameMode;
  region: Region | null;
  log: EditCommand[] | undefined;
}

export interface WorkerContext {
  map: MapHandle;
  memory: WebAssembly.Memory;
  post: Post;
}

const NOTICE = 'Save could not be restored';

function replayed(engine: Engine, log: EditCommand[]): CommandResult[] {
  const results: unknown = engine.replay(log);
  if (!isCommandResults(results)) {
    throw new Error('Invalid replay results from wasm');
  }
  return results;
}

function tryReplay(ctx: WorkerContext, request: EngineRequest) {
  const engine = Engine.fromMap(ctx.map, request.config);
  try {
    return { engine, results: replayed(engine, request.log ?? []), restored: true };
  } catch {
    engine.free();
    const fresh = Engine.fromMap(ctx.map, request.config);
    return { engine: fresh, results: [], restored: false };
  }
}

function postReady(ctx: WorkerContext, engine: Engine, request: EngineRequest): void {
  const meta: unknown = engine.meta();
  if (!isMapMeta(meta)) {
    throw new Error('Invalid map meta from wasm');
  }
  const geometry = collectGeometry(engine);
  const { session, mode, region } = request;
  const ready: ReadyMessage = { type: 'ready', session, mode, region, meta, ...geometry };
  ctx.post(ready, transferList(geometry));
}

export function startEngine(ctx: WorkerContext, request: EngineRequest): EngineSession {
  const { engine, results, restored } = tryReplay(ctx, request);
  const live = new EngineSession(request.session, engine, ctx.memory, ctx.post);
  const delta = live.deltaFor(results);
  postReady(ctx, engine, request);
  live.announce(delta);
  live.postSignals();
  if (!restored) {
    ctx.post({ type: 'notice', session: request.session, text: NOTICE }, []);
  }
  return live;
}
