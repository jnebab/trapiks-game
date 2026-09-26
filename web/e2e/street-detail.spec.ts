import { expect, test, type Page } from '@playwright/test';
import { PNG } from 'pngjs';

const debug = '#debug';
const WHITE_MIN = 245;

async function waitForMarkings(page: Page): Promise<void> {
  const overlay = page.locator(debug);
  await expect
    .poll(
      async () => {
        const built = Number(await overlay.getAttribute('data-markings-built'));
        const visible = Number(await overlay.getAttribute('data-tiles-visible'));
        return built === visible && visible > 0;
      },
      { timeout: 20_000 },
    )
    .toBe(true);
}

function pixelAt(png: PNG, x: number, y: number): number[] {
  const i = (y * png.width + x) * 4;
  return [png.data[i] ?? 0, png.data[i + 1] ?? 0, png.data[i + 2] ?? 0];
}

async function capture(page: Page, url: string, path: string): Promise<PNG> {
  await page.goto(url);
  await waitForMarkings(page);
  await page.waitForTimeout(200);
  return PNG.sync.read(await page.screenshot({ path }));
}

test('signalized arterial junction shows markings and fillets', async ({ page }, testInfo) => {
  const url = '/?map=synthetic&vph=0&cx=750&cy=750&z=12';
  const png = await capture(page, url, testInfo.outputPath('street-detail.png'));
  for (const row of [359, 360]) {
    const [r, g, b] = pixelAt(png, 1000, row);
    expect(r).toBeGreaterThan(200);
    expect(g).toBeGreaterThan(170);
    expect(b).toBeLessThan(120);
  }
  for (const channel of pixelAt(png, 727, 272)) {
    expect(channel).toBeGreaterThanOrEqual(WHITE_MIN);
  }
});

test('residential approaches show yield lines', async ({ page }, testInfo) => {
  const url = '/?map=synthetic&vph=0&cx=150&cy=750&z=12';
  await capture(page, url, testInfo.outputPath('street-residential.png'));
});
