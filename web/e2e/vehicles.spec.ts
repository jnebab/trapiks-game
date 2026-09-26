import { expect, test, type Page } from '@playwright/test';

const debug = '#debug';

async function readTick(page: Page): Promise<number> {
  return Number(await page.locator(debug).getAttribute('data-tick'));
}

async function tickAdvanceOver(page: Page, ms: number): Promise<number> {
  const start = await readTick(page);
  await page.waitForTimeout(ms);
  return (await readTick(page)) - start;
}

test('runs and draws vehicles with speed controls', async ({ page }, testInfo) => {
  test.setTimeout(60_000);
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') {
      errors.push(message.text());
    }
  });
  page.on('pageerror', (error) => {
    errors.push(error.message);
  });
  await page.goto('/?map=synthetic&vph=6000#/sandbox');
  await expect
    .poll(async () => Number(await page.locator(debug).getAttribute('data-vehicles')), {
      timeout: 10_000,
    })
    .toBeGreaterThan(0);
  const atOne = await tickAdvanceOver(page, 3_000);
  expect(atOne).toBeGreaterThanOrEqual(20);
  await page.keyboard.press('5');
  await expect(page.locator(debug)).toHaveAttribute('data-speed', 'max');
  const atMax = await tickAdvanceOver(page, 3_000);
  expect(atMax).toBeGreaterThan(atOne);
  await page.keyboard.press('Space');
  await expect(page.locator(debug)).toHaveAttribute('data-speed', '0');
  await page.waitForTimeout(700);
  const t1 = await readTick(page);
  await page.waitForTimeout(1_000);
  const t2 = await readTick(page);
  expect(t2).toBe(t1);
  expect(errors).toEqual([]);
  await page.screenshot({ path: testInfo.outputPath('vehicles.png') });
});

test('street zoom shows jeepneys and buses', async ({ page }, testInfo) => {
  test.setTimeout(150_000);
  await page.goto('/?map=synthetic&cx=1500&cy=1500&z=4#/challenge/downtown');
  await page.waitForTimeout(20_000);
  await page.keyboard.press('4');
  await page.waitForTimeout(40_000);
  await page.keyboard.press('Space');
  await page.waitForTimeout(1_000);
  await page.screenshot({ path: testInfo.outputPath('vehicle-kinds.png') });
});
