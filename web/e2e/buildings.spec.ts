import { expect, test, type Page } from '@playwright/test';
import { PNG } from 'pngjs';

const TOLERANCE = 6;
const HOUSE = [0x8f, 0xd1, 0x6a] as const;
const HOUSE_SHADE = [0x62, 0xa8, 0x3e] as const;

async function waitForBuildings(page: Page): Promise<void> {
  const overlay = page.locator('#debug');
  await expect
    .poll(
      async () => {
        const built = Number(await overlay.getAttribute('data-buildings-built'));
        const visible = Number(await overlay.getAttribute('data-tiles-visible'));
        return built === visible && visible > 0;
      },
      { timeout: 20_000 },
    )
    .toBe(true);
}

function countColour(png: PNG, colour: readonly number[]): number {
  let count = 0;
  for (let i = 0; i < png.data.length; i += 4) {
    const close = colour.every((c, k) => Math.abs((png.data[i + k] ?? 0) - c) <= TOLERANCE);
    count += close ? 1 : 0;
  }
  return count;
}

test('street zoom shows gabled houses', async ({ page }, testInfo) => {
  await page.goto('/?map=synthetic&vph=0&cx=975&cy=450&z=6#/sandbox');
  await waitForBuildings(page);
  await page.waitForTimeout(300);
  const png = PNG.sync.read(await page.screenshot({ path: testInfo.outputPath('buildings.png') }));
  expect(countColour(png, HOUSE)).toBeGreaterThan(50);
  expect(countColour(png, HOUSE_SHADE)).toBeGreaterThan(50);
});
