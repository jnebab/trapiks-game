import { expect, test, type Page } from '@playwright/test';
import { PNG } from 'pngjs';

const debug = '#debug';
const SAMPLE = { x: 640, y: 336 } as const;
const GROUND = [0xee, 0xed, 0xe6] as const;
const GROUND_TOLERANCE = 6;
const WHITE_MIN = 245;
const CYAN_MARGIN = 20;

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

async function numberAttr(page: Page, selector: string, name: string): Promise<number> {
  return Number(await page.locator(selector).getAttribute(name));
}

async function waitForTiles(page: Page): Promise<void> {
  await expect
    .poll(
      async () => {
        const built = await numberAttr(page, debug, 'data-tiles-built');
        const visible = await numberAttr(page, debug, 'data-tiles-visible');
        return built === visible && visible > 0;
      },
      { timeout: 20_000 },
    )
    .toBe(true);
}

async function samplePixel(page: Page): Promise<number[]> {
  const png = PNG.sync.read(await page.screenshot());
  const i = (SAMPLE.y * png.width + SAMPLE.x) * 4;
  return [png.data[i] ?? 0, png.data[i + 1] ?? 0, png.data[i + 2] ?? 0];
}

function isCyan([r = 0, , b = 0]: number[]): boolean {
  return b > r + CYAN_MARGIN;
}

function isWhite(pixel: number[]): boolean {
  return pixel.every((channel) => channel >= WHITE_MIN);
}

function isGround(pixel: number[]): boolean {
  return pixel.every((channel, i) => Math.abs(channel - (GROUND[i] ?? 0)) <= GROUND_TOLERANCE);
}

async function spent(page: Page): Promise<number> {
  return numberAttr(page, '#budget', 'data-spent');
}

test('hover, delete and undo a residential road', async ({ page }, testInfo) => {
  test.setTimeout(60_000);
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=0&cx=975&cy=450&z=16#/sandbox');
  await waitForTiles(page);
  await page.mouse.move(640, 360);
  await expect.poll(async () => isCyan(await samplePixel(page)), { timeout: 10_000 }).toBe(true);
  await page.screenshot({ path: testInfo.outputPath('edit-hover.png') });
  await page.keyboard.press('Delete');
  await expect.poll(async () => spent(page), { timeout: 10_000 }).toBeGreaterThan(0);
  await expect.poll(async () => isGround(await samplePixel(page)), { timeout: 10_000 }).toBe(true);
  await page.screenshot({ path: testInfo.outputPath('edit-deleted.png') });
  await page.keyboard.press('Control+z');
  await expect.poll(async () => spent(page), { timeout: 10_000 }).toBe(0);
  await expect
    .poll(
      async () => {
        const pixel = await samplePixel(page);
        return isWhite(pixel) || isCyan(pixel);
      },
      { timeout: 10_000 },
    )
    .toBe(true);
  await page.screenshot({ path: testInfo.outputPath('edit-undone.png') });
  expect(errors).toEqual([]);
});
