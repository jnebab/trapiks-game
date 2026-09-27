import { expect, test, type Page } from '@playwright/test';

const debug = '#debug';

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

async function numberAttr(page: Page, name: string): Promise<number> {
  return Number(await page.locator(debug).getAttribute(name));
}

async function waitForTiles(page: Page): Promise<void> {
  await expect
    .poll(
      async () => {
        const built = await numberAttr(page, 'data-tiles-built');
        const visible = await numberAttr(page, 'data-tiles-visible');
        return built === visible && visible > 0;
      },
      { timeout: 20_000 },
    )
    .toBe(true);
}

test('skyway over a crossing casts shadows', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=0&cx=2250&cy=2250&z=5#/sandbox');
  await waitForTiles(page);
  await expect.poll(async () => numberAttr(page, 'data-shadow-pieces')).toBeGreaterThan(0);
  await page.waitForTimeout(300);
  await page.screenshot({ path: testInfo.outputPath('depth-ramp.png') });
  expect(errors).toEqual([]);
});

function layerCount(summary: string | null, layer: number): number {
  const entry = (summary ?? '').split(',').find((part) => part.startsWith(`${String(layer)}:`));
  return Number(entry?.split(':')[1] ?? 0);
}

test('ground vehicles draw in the layer below the skyway', async ({ page }, testInfo) => {
  test.setTimeout(60_000);
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=6000&cx=2250&cy=2250&z=6#/sandbox');
  await waitForTiles(page);
  await page.keyboard.press('5');
  await expect
    .poll(
      async () => {
        const summary = await page.locator(debug).getAttribute('data-vehicle-layers');
        return layerCount(summary, 0) > 0 && layerCount(summary, 1) > 0;
      },
      { timeout: 30_000 },
    )
    .toBe(true);
  await page.keyboard.press('Space');
  await page.waitForTimeout(300);
  await page.screenshot({ path: testInfo.outputPath('depth-under-skyway.png') });
  expect(errors).toEqual([]);
});

test('signal junction shows live pills', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=0&cx=750&cy=750&z=12#/sandbox');
  await waitForTiles(page);
  await expect.poll(async () => numberAttr(page, 'data-signal-pills')).toBeGreaterThanOrEqual(4);
  await page.keyboard.press('5');
  await expect
    .poll(async () => numberAttr(page, 'data-signal-updates'), { timeout: 10_000 })
    .toBeGreaterThanOrEqual(2);
  await page.screenshot({ path: testInfo.outputPath('depth-signals.png') });
  expect(errors).toEqual([]);
});

test('vehicles use the polished sprite', async ({ page }, testInfo) => {
  test.setTimeout(60_000);
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=6000&cx=750&cy=900&z=10#/sandbox');
  await waitForTiles(page);
  await page.keyboard.press('5');
  await expect
    .poll(async () => numberAttr(page, 'data-vehicles'), { timeout: 20_000 })
    .toBeGreaterThanOrEqual(400);
  await page.keyboard.press('Space');
  await page.waitForTimeout(300);
  await page.screenshot({ path: testInfo.outputPath('depth-vehicles.png') });
  expect(errors).toEqual([]);
});
