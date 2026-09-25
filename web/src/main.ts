import './hud/styles.css';
import '@fontsource/lexend/500.css';
import { renderTitle } from './hud/title';
import { startSim } from './sim/client';

const root = document.querySelector<HTMLElement>('#app');
if (!root) {
  throw new Error('Missing #app root element');
}

startSim((info) => {
  renderTitle(root, info);
});
