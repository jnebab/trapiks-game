import './hud/styles.css';
import '@fontsource/lexend/500.css';
import '@fontsource/lexend/700.css';
import { runGame } from './app/game';

const DEFAULT_MAP = 'metro-manila';
const MAP_NAME = /^[a-z0-9-]+$/;

function mapName(): string {
  const requested = new URLSearchParams(window.location.search).get('map');
  return requested !== null && MAP_NAME.test(requested) ? requested : DEFAULT_MAP;
}

const root = document.querySelector<HTMLElement>('#app');
if (!root) {
  throw new Error('Missing #app root element');
}

void runGame(root, mapName());
