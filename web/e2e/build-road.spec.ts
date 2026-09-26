import { expect, test, type Page } from '@playwright/test';
import { PNG } from 'pngjs';

const debug = '#debug';
const tooltip = '#road-tooltip';
const CENTRE = { x: 640, y: 360 } as const;
const CORNER = { x: 20, y: 600 } as const;
const START = { x: 415, y: 135 } as const;
const VIA = { x: 700, y: 300 } as const;
const END = { x: 865, y: 585 } as const;
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

async function drawTo(page: Page, point: { x: number; y: number }): Promise<void> {
  await page.mouse.move(point.x - 20, point.y - 20);
  await page.mouse.move(point.x, point.y, { steps: 4 });
  await expect(page.locator(tooltip)).toContainText('₱', { timeout: 10_000 });
}

test('draw, undo and draw an elevated curved road', async ({ page }, testInfo) => {
  test.setTimeout(90_000);
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=0&cx=375&cy=375&z=3#/sandbox');
  await waitForTiles(page);
  await page.keyboard.press('r');
  await expect(page.locator('#tool-road')).toHaveAttribute('aria-pressed', 'true');
  await page.mouse.click(START.x, START.y);
  await drawTo(page, END);
  const before = await spent(page);
  await page.mouse.click(END.x, END.y);
  await expect.poll(async () => spent(page), { timeout: 10_000 }).toBeGreaterThan(before);
  await expect.poll(async () => isWhite(await centrePixel(page)), { timeout: 10_000 }).toBe(true);
  await page.keyboard.press('Control+z');
  await expect.poll(async () => isGround(await centrePixel(page)), { timeout: 10_000 }).toBe(true);
  await page.locator('#road-curve').click();
  await page.getByRole('button', { name: 'Raise' }).click();
  await expect(page.locator('#elevation-value')).toHaveText('1');
  const shadows = await numberAttr(page, debug, 'data-shadow-pieces');
  await page.mouse.click(START.x, START.y);
  await page.mouse.click(VIA.x, VIA.y);
  await drawTo(page, END);
  await expect(page.locator(tooltip)).toContainText('Elevation: 1');
  await page.screenshot({ path: testInfo.outputPath('build-road-drawing.png') });
  await page.mouse.click(END.x, END.y);
  await expect
    .poll(async () => numberAttr(page, debug, 'data-shadow-pieces'), { timeout: 10_000 })
    .toBeGreaterThan(shadows);
  await page.mouse.move(CORNER.x, CORNER.y);
  await page.screenshot({ path: testInfo.outputPath('build-road.png') });
  expect(errors).toEqual([]);
});
