import { expect, test, type Page } from '@playwright/test';

const BASE = 'http://localhost:4174/trapiks-game/';

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

test('title screen loads under the Pages base path', async ({ page }) => {
  const errors = collectErrors(page);
  await page.goto(`${BASE}?map=synthetic#/`);
  await expect(page.getByRole('button', { name: 'Challenges' })).toBeVisible({ timeout: 20_000 });
  await page.getByRole('button', { name: 'Challenges' }).click();
  await expect(page.locator('.challenge-card').first()).toBeVisible({ timeout: 20_000 });
  expect(errors).toEqual([]);
});

test('sandbox runs vehicles under the Pages base path', async ({ page }) => {
  const errors = collectErrors(page);
  await page.goto(`${BASE}?map=synthetic&vph=20000#/sandbox`);
  await expect
    .poll(async () => Number(await page.locator('#debug').getAttribute('data-vehicles')), {
      timeout: 30_000,
    })
    .toBeGreaterThan(0);
  expect(errors).toEqual([]);
});
