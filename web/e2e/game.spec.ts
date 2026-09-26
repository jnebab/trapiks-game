import { expect, test, type Page } from '@playwright/test';

const HOME = '/?map=synthetic&cx=1500&cy=750&z=6#/';
const debug = '#debug';
const CENTRE = { x: 640, y: 360 } as const;
const CORNER = { x: 20, y: 600 } as const;
const SHOTS = process.env.GAME_SHOTS;

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

async function spent(page: Page): Promise<number> {
  return numberAttr(page, '#budget', 'data-spent');
}

async function waitForTiles(page: Page): Promise<void> {
  await expect
    .poll(
      async () => {
        const built = await numberAttr(page, debug, 'data-tiles-built');
        const visible = await numberAttr(page, debug, 'data-tiles-visible');
        return built === visible && visible > 0;
      },
      { timeout: 30_000 },
    )
    .toBe(true);
}

async function shot(page: Page, name: string, outputPath: (name: string) => string) {
  await page.mouse.move(CORNER.x, CORNER.y);
  const path = SHOTS === undefined ? outputPath(name) : `${SHOTS}/${name}`;
  await page.screenshot({ path });
}

async function openChallenge(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Challenges' }).click();
  await page.locator('.challenge-card', { hasText: 'Downtown Box' }).click();
  const objective = page.locator('#objective');
  await expect(objective).toContainText('Measuring baseline', { timeout: 20_000 });
  await expect
    .poll(async () => objective.textContent(), { timeout: 90_000 })
    .toContain('baseline ');
  await expect(objective).not.toContainText('Measuring');
}

async function buildFlyover(page: Page): Promise<number> {
  await waitForTiles(page);
  const shadows = await numberAttr(page, debug, 'data-shadow-pieces');
  const before = await spent(page);
  await page.mouse.click(CENTRE.x, CENTRE.y);
  await expect(page.locator('#inspector')).toBeVisible({ timeout: 10_000 });
  await page
    .getByRole('button', { name: /^Build flyover/ })
    .first()
    .click();
  await expect.poll(async () => spent(page), { timeout: 10_000 }).toBeGreaterThan(before);
  return shadows;
}

async function evaluate(page: Page): Promise<void> {
  await page.locator('#evaluate').click();
  const result = page.locator('#result');
  await expect(result).toBeVisible({ timeout: 120_000 });
  await expect(result.locator('.stars')).toBeVisible();
  await expect(result.locator('#improvement')).toHaveText(/-?\d+\.\d %/);
}

async function continueSave(page: Page, spentBefore: number, shadows: number): Promise<void> {
  await page.goto(HOME);
  await page.reload();
  await expect(page.locator('#continue')).toBeVisible({ timeout: 20_000 });
  await page.locator('#continue').click();
  await expect.poll(async () => spent(page), { timeout: 30_000 }).toBe(spentBefore);
  await waitForTiles(page);
  await expect
    .poll(async () => numberAttr(page, debug, 'data-shadow-pieces'), { timeout: 20_000 })
    .toBeGreaterThan(shadows);
}

async function sandboxTraffic(page: Page): Promise<void> {
  await page.goto('/?map=synthetic&cx=1500&cy=750&z=6#/sandbox');
  await waitForTiles(page);
  const before = await numberAttr(page, debug, 'data-vehicles');
  await page.locator('#demand').fill('20000');
  await expect
    .poll(async () => numberAttr(page, debug, 'data-vehicles'), { timeout: 15_000 })
    .toBeGreaterThan(before);
  await page.locator('#demand').blur();
  await page.keyboard.press('t');
  await expect(page.locator(debug)).toHaveAttribute('data-traffic', '1');
}

test('title, challenge, evaluation, continue and sandbox', async ({ page }, testInfo) => {
  test.setTimeout(300_000);
  const errors = collectErrors(page);
  const out = (name: string): string => testInfo.outputPath(name);
  await page.goto(HOME);
  await expect(page.locator('#title')).toBeVisible({ timeout: 20_000 });
  await expect(page.locator('.attribution')).toHaveText('Synthetic map');
  await shot(page, 'title.png', out);
  await openChallenge(page);
  const shadows = await buildFlyover(page);
  const afterFlyover = await spent(page);
  await shot(page, 'challenge.png', out);
  await evaluate(page);
  await shot(page, 'result.png', out);
  await continueSave(page, afterFlyover, shadows);
  await sandboxTraffic(page);
  await page.waitForTimeout(2_500);
  await shot(page, 'sandbox-traffic.png', out);
  expect(errors).toEqual([]);
});
