import type { DeltaGeometry, Engine } from '../wasm/pkg/trapiks_sim_wasm.js';
import type { CommandResult } from '../generated/CommandResult';
import { deltaColumnNames, type DeltaArrays, type DeltaColumns } from '../sim/delta-arrays';

export interface ChangedIds {
  roads: Uint32Array;
  nodes: Uint32Array;
}

export function changedIds(results: readonly CommandResult[]): ChangedIds | null {
  const roads: number[] = [];
  const nodes: number[] = [];
  let succeeded = false;
  for (const result of results) {
    if ('Ok' in result.outcome) {
      succeeded = true;
      roads.push(...result.outcome.Ok.changed_roads);
      nodes.push(...result.outcome.Ok.changed_nodes);
    }
  }
  if (!succeeded) {
    return null;
  }
  return { roads: Uint32Array.from(new Set(roads)), nodes: Uint32Array.from(new Set(nodes)) };
}

function columnsOf(g: DeltaGeometry): DeltaColumns {
  const entries = deltaColumnNames.map((name) => [name, g[name]] as const);
  return Object.fromEntries(entries) as unknown as DeltaColumns;
}

export function collectDelta(engine: Engine, ids: ChangedIds): DeltaArrays {
  const g = engine.delta(ids.roads, ids.nodes);
  const delta: DeltaArrays = { roadCount: g.roadCount, nodeCount: g.nodeCount, ...columnsOf(g) };
  g.free();
  return delta;
}
