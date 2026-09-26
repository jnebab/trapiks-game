import type { NodeInspection } from '../generated/NodeInspection';
import type { RoadInspection } from '../generated/RoadInspection';
import { isRecord } from './values';

export type InspectTarget = { road: number } | { node: number };

export type Inspection = { road: RoadInspection } | { node: NodeInspection };

export interface InspectMessage {
  type: 'inspect';
  id: number;
  target: InspectTarget;
}

export interface InspectionMessage {
  type: 'inspection';
  session: number;
  id: number;
  target: Inspection | null;
}

function isNumber(value: unknown): value is number {
  return typeof value === 'number';
}

export function isInspectTarget(value: unknown): value is InspectTarget {
  if (!isRecord(value)) {
    return false;
  }
  const keys = Object.keys(value);
  const key = keys[0];
  return keys.length === 1 && (key === 'road' || key === 'node') && isNumber(value[key]);
}

export function isRoadInspection(value: unknown): value is RoadInspection {
  if (!isRecord(value)) {
    return false;
  }
  const numbers = ['road', 'length_m', 'lanes_forward', 'lanes_backward', 'speed_kph', 'layer'];
  return (
    numbers.every((key) => isNumber(value[key])) &&
    typeof value.class === 'string' &&
    typeof value.deleted === 'boolean'
  );
}

function isTurnInfo(value: unknown): boolean {
  return (
    isRecord(value) &&
    isNumber(value.from_road) &&
    isNumber(value.to_road) &&
    typeof value.kind === 'string' &&
    typeof value.allowed === 'boolean'
  );
}

function isSignalInfo(value: unknown): boolean {
  return (
    isRecord(value) &&
    isNumber(value.phase_count) &&
    isListOf(value.greens_s, isNumber) &&
    isNumber(value.offset_s)
  );
}

function isPair(value: unknown): boolean {
  return isListOf(value, isNumber) && Array.isArray(value) && value.length === 2;
}

function isListOf(value: unknown, guard: (item: unknown) => boolean): boolean {
  return Array.isArray(value) && value.every(guard);
}

function hasNodeLists(value: Record<string, unknown>): boolean {
  return (
    isListOf(value.roads, isNumber) &&
    isListOf(value.turns, isTurnInfo) &&
    isListOf(value.flyover_pairs, isPair)
  );
}

export function isNodeInspection(value: unknown): value is NodeInspection {
  if (!isRecord(value)) {
    return false;
  }
  return (
    isNumber(value.node) &&
    typeof value.control === 'string' &&
    (value.signal === null || isSignalInfo(value.signal)) &&
    hasNodeLists(value)
  );
}

export function isInspection(value: unknown): value is Inspection {
  if (!isRecord(value)) {
    return false;
  }
  if ('road' in value) {
    return Object.keys(value).length === 1 && isRoadInspection(value.road);
  }
  return Object.keys(value).length === 1 && isNodeInspection(value.node);
}

export function isInspectMessage(value: Record<string, unknown>): boolean {
  return isNumber(value.id) && isInspectTarget(value.target);
}

export function isInspectionMessage(value: Record<string, unknown>): boolean {
  return isNumber(value.id) && (value.target === null || isInspection(value.target));
}
