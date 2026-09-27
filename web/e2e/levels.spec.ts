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

async function waitForTiles(page: Page): Promise<void> {
  await expect
    .poll(
      async () => {
        const built = await page.locator(debug).getAttribute('data-tiles-built');
        const visible = await page.locator(debug).getAttribute('data-tiles-visible');
        return built === visible && Number(visible) > 0;
      },
      { timeout: 20_000 },
    )
    .toBe(true);
}

function connectionTotal(summary: string | null): number {
  return (summary ?? '0/0/0')
    .split('/')
    .slice(0, 2)
    .reduce((sum, part) => sum + Number(part), 0);
}

test('L cycles the level view', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=0&cx=2250&cy=2250&z=5#/sandbox');
  await waitForTiles(page);
  await expect(page.locator(debug)).toHaveAttribute('data-level-view', 'all');
  const views = ['see-through', 'ground', 'elevated', 'all'];
  for (const view of views) {
    await page.keyboard.press('l');
    await expect(page.locator(debug)).toHaveAttribute('data-level-view', view);
    await expect(page.locator(`#levels [data-level="${view}"]`)).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    await page.screenshot({ path: testInfo.outputPath(`levels-${view}.png`) });
  }
  expect(errors).toEqual([]);
});

test('hovering a road reports its connections', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  await page.goto('/?map=synthetic&vph=0&cx=2250&cy=2250&z=5#/sandbox');
  await waitForTiles(page);
  const box = await page.locator('canvas').boundingBox();
  const width = box?.width ?? 0;
  const height = box?.height ?? 0;
  await page.mouse.move(width / 2 + 1, height / 2 + 1);
  await expect
    .poll(async () => connectionTotal(await page.locator(debug).getAttribute('data-connections')))
    .toBeGreaterThan(0);
  await page.screenshot({ path: testInfo.outputPath('connections-hover.png') });
  expect(errors).toEqual([]);
});
