import type { EngineInfo } from '../generated/EngineInfo';

export interface ReadyMessage {
  type: 'ready';
  info: EngineInfo;
}

export type WorkerMessage = ReadyMessage;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

export function isEngineInfo(value: unknown): value is EngineInfo {
  if (!isRecord(value)) {
    return false;
  }
  return typeof value.name === 'string' && typeof value.map_format_version === 'number';
}

export function isWorkerMessage(value: unknown): value is WorkerMessage {
  if (!isRecord(value)) {
    return false;
  }
  return value.type === 'ready' && isEngineInfo(value.info);
}
