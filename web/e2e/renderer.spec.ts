import { expect, test, type Page } from '@playwright/test';
import { PNG } from 'pngjs';

const debug = '#debug';
const SCALE = 16;
const WHITE_MIN = 245;
const OFF_CENTER_LINE = SCALE;

function collectErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') {
      errors.push(message.text());
    }
  });
  page.on('pageerror', (error) => {
    errors.push(error.message);
  });
  return errors;
}

async function tileCounts(page: Page): Promise<[number, number]> {
  const overlay = page.locator(debug);
  const built = Number(await overlay.getAttribute('data-tiles-built'));
  const visible = Number(await overlay.getAttribute('data-tiles-visible'));
  return [built, visible];
}

async function waitForTiles(page: Page): Promise<void> {
  await expect
    .poll(async () => {
      const [built, visible] = await tileCounts(page);
      return built === visible && visible > 0;
    })
    .toBe(true);
}

function pixelAt(png: PNG, x: number, y: number): number[] {
  const i = (Math.round(y) * png.width + Math.round(x)) * 4;
  return [png.data[i] ?? 0, png.data[i + 1] ?? 0, png.data[i + 2] ?? 0];
}

test('tile borders and junctions are seamless', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=0&cx=1024&cy=450&z=16');
  await expect(page.locator(debug)).toHaveAttribute('data-band', 'detail');
  await waitForTiles(page);
  const path = testInfo.outputPath('renderer-boundary.png');
  const png = PNG.sync.read(await page.screenshot({ path }));
  const [cx, cy] = [png.width / 2, png.height / 2 - OFF_CENTER_LINE];
  const samples: [number, number][] = [
    [cx - 3, cy],
    [cx + 3, cy],
    [cx + 22.625 * SCALE, cy],
  ];
  for (const [x, y] of samples) {
    for (const channel of pixelAt(png, x, y)) {
      expect(channel).toBeGreaterThanOrEqual(WHITE_MIN);
    }
  }
  expect(errors).toEqual([]);
});

test('city zoom shows the city band', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=0&z=0.1');
  await expect(page.locator(debug)).toHaveAttribute('data-band', 'city');
  await waitForTiles(page);
  await page.screenshot({ path: testInfo.outputPath('renderer-city.png') });
  expect(errors).toEqual([]);
});
