import type { Engine } from '../wasm/pkg/trapiks_sim_wasm.js';
import {
  isNodeInspection,
  isRoadInspection,
  type InspectTarget,
  type Inspection,
} from '../sim/inspect-values';

function checked<T>(value: unknown, guard: (value: unknown) => value is T): T | null {
  if (value === null || value === undefined) {
    return null;
  }
  if (!guard(value)) {
    throw new Error('Invalid inspection from wasm');
  }
  return value;
}

export function inspect(engine: Engine, target: InspectTarget): Inspection | null {
  if ('road' in target) {
    const road = checked(engine.inspectRoad(target.road), isRoadInspection);
    return road === null ? null : { road };
  }
  const node = checked(engine.inspectNode(target.node), isNodeInspection);
  return node === null ? null : { node };
}
