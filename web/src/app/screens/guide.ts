import { el } from '../../hud/dom';
import { GUIDE_SECTIONS, type GuideSection, type GuideTable } from '../guide-content';
import type { GameContext, Screen } from '../screen';
import { button } from './parts';

function tableOf(table: GuideTable): HTMLTableElement {
  const element = el('table', 'guide-table');
  const head = el('tr');
  head.append(...table.head.map((text) => el('th', undefined, text)));
  const rows = table.rows.map((row) => {
    const line = el('tr');
    line.append(...row.map((text) => el('td', undefined, text)));
    return line;
  });
  element.append(head, ...rows);
  return element;
}

function sectionOf(section: GuideSection): HTMLElement {
  const element = el('section', 'guide-section');
  element.append(
    el('h2', undefined, section.heading),
    ...section.paragraphs.map((text) => el('p', undefined, text)),
  );
  if (section.table !== undefined) {
    element.appendChild(tableOf(section.table));
  }
  return element;
}

export function createGuideScreen(ctx: GameContext): Screen {
  ctx.showBackground();
  const element = el('div', 'guide-screen');
  element.id = 'guide';
  const card = el('div', 'guide-card');
  const back = button('← Back', 'chip', () => {
    ctx.navigate({ name: 'title' });
  });
  back.id = 'back';
  const header = el('div', 'guide-header');
  header.append(back, el('h1', 'guide-title', 'How to play'));
  card.append(header, ...GUIDE_SECTIONS.map(sectionOf));
  element.appendChild(card);
  return { element };
}
