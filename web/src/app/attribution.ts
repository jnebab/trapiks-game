import { el } from '../hud/dom';

const SYNTHETIC = 'synthetic';
const OSM_TEXT = '© OpenStreetMap contributors (ODbL)';
const OSM_URL = 'https://www.openstreetmap.org/copyright';

export function attributionText(mapName: string): string {
  return mapName === SYNTHETIC ? 'Synthetic map' : OSM_TEXT;
}

export function createAttribution(mapName: string): HTMLElement {
  if (mapName === SYNTHETIC) {
    return el('div', 'chip attribution', attributionText(mapName));
  }
  const link = el('a', 'chip attribution', OSM_TEXT);
  link.href = OSM_URL;
  link.target = '_blank';
  link.rel = 'noopener';
  return link;
}
