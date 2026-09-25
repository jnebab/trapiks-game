import type { EngineInfo } from '../generated/EngineInfo';
import { el } from './dom';

export function renderTitle(root: HTMLElement, info: EngineInfo): void {
  const heading = el('h1', undefined, 'Trapiks');
  const chip = el('span', 'chip', `map format v${String(info.map_format_version)}`);
  root.replaceChildren(heading, chip);
}
