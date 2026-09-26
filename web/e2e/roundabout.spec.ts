import { expect, test, type Page } from '@playwright/test';
import { PNG } from 'pngjs';

const debug = '#debug';
const inspector = '#inspector';
const CENTRE = { x: 640, y: 360 } as const;
const CORNER = { x: 20, y: 600 } as const;
const GROUND = [0xee, 0xed, 0xe6];
const GROUND_TOLERANCE = 6;
const WHITE_MIN = 245;

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
  return Number(await page.locator(selector).first().getAttribute(name));
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

async function centrePixel(page: Page): Promise<number[]> {
  await page.mouse.move(CORNER.x, CORNER.y);
  const png = PNG.sync.read(await page.screenshot());
  const i = (CENTRE.y * png.width + CENTRE.x) * 4;
  return [png.data[i] ?? 0, png.data[i + 1] ?? 0, png.data[i + 2] ?? 0];
}

function isGround(pixel: number[]): boolean {
  return pixel.every((channel, i) => Math.abs(channel - (GROUND[i] ?? 0)) <= GROUND_TOLERANCE);
}

function isWhite(pixel: number[]): boolean {
  return pixel.every((channel) => channel >= WHITE_MIN);
}

async function spent(page: Page): Promise<number> {
  return numberAttr(page, '#budget', 'data-spent');
}

test('build and undo a roundabout', async ({ page }, testInfo) => {
  test.setTimeout(90_000);
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=0&cx=150&cy=750&z=10#/sandbox');
  await waitForTiles(page);
  await page.mouse.click(CENTRE.x, CENTRE.y);
  await expect(page.locator(inspector)).toBeVisible({ timeout: 10_000 });
  const before = await spent(page);
  await page.getByRole('button', { name: 'Large', exact: true }).click();
  await expect.poll(async () => spent(page), { timeout: 10_000 }).toBeGreaterThan(before);
  await expect.poll(async () => isGround(await centrePixel(page)), { timeout: 10_000 }).toBe(true);
  await page.screenshot({ path: testInfo.outputPath('roundabout.png') });
  await page.locator('#undo').click();
  await expect.poll(async () => isWhite(await centrePixel(page)), { timeout: 10_000 }).toBe(true);
  expect(errors).toEqual([]);
});
