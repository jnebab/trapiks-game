import { expect, test, type Page } from '@playwright/test';

const HOME = '/?map=synthetic&cx=1500&cy=750&z=6#/';
const SHOTS = process.env.GUIDE_SHOTS;

async function shot(page: Page, name: string): Promise<void> {
  if (SHOTS !== undefined) {
    await page.screenshot({ path: `${SHOTS}/${name}` });
  }
}

test('how to play opens the guide and back returns to the title', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto(HOME);
  await expect(page.locator('#title')).toBeVisible({ timeout: 20_000 });
  await shot(page, 'title-guide-button.png');
  await page.locator('#how-to-play').click();
  await expect(page.locator('#guide')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'How to play' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Road tool (R)' })).toBeAttached();
  await shot(page, 'guide-1280.png');
  await page.setViewportSize({ width: 390, height: 844 });
  await shot(page, 'guide-phone.png');
  await page.locator('#back').click();
  await expect(page.locator('#title')).toBeVisible();
});

test('first challenge visit shows a dismissible hint', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto(HOME);
  await page.getByRole('button', { name: 'Challenges' }).click();
  await page.locator('.challenge-card', { hasText: 'Downtown Box' }).click();
  const hint = page.locator('#first-hint');
  await expect(hint).toBeVisible({ timeout: 20_000 });
  await expect(hint.getByRole('link', { name: 'How to play' })).toHaveAttribute('href', '#/guide');
  await expect(page.locator('#help')).toBeVisible();
  await shot(page, 'challenge-hint.png');
  await hint.getByRole('button', { name: 'Dismiss hint' }).click();
  await expect(hint).toHaveCount(0);
  await page.reload();
  await expect(page.locator('#objective')).toBeVisible({ timeout: 20_000 });
  await expect(page.locator('#first-hint')).toHaveCount(0);
});
