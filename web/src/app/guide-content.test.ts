import { describe, expect, it } from 'vitest';
import { GUIDE_SECTIONS } from './guide-content';

describe('GUIDE_SECTIONS', () => {
  it('has unique headings with text', () => {
    const headings = GUIDE_SECTIONS.map((section) => section.heading);
    expect(new Set(headings).size).toBe(headings.length);
    for (const section of GUIDE_SECTIONS) {
      expect(section.paragraphs.length).toBeGreaterThan(0);
    }
  });

  it('keeps table rows as wide as their header', () => {
    for (const section of GUIDE_SECTIONS) {
      for (const row of section.table?.rows ?? []) {
        expect(row.length).toBe(section.table?.head.length);
      }
    }
  });
});
