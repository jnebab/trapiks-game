import { expect, test } from '@playwright/test';

test('renders the synthetic map without errors', async ({ page }, testInfo) => {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') {
      errors.push(message.text());
    }
  });
  page.on('pageerror', (error) => {
    errors.push(error.message);
  });
  await page.goto('/?map=synthetic');
  const debug = page.locator('#debug[data-roads]');
  await expect(debug).toBeAttached();
  const roads = Number(await debug.getAttribute('data-roads'));
  expect(roads).toBeGreaterThan(0);
  expect(errors).toEqual([]);
  const screenshot = testInfo.outputPath('smoke.png');
  await page.screenshot({ path: screenshot });
});
