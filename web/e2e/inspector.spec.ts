import { expect, test, type Page } from '@playwright/test';

const debug = '#debug';
const inspector = '#inspector';
const CENTRE = { x: 640, y: 360 } as const;
const CORNER = { x: 20, y: 600 } as const;

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
      { timeout: 20_000 },
    )
    .toBe(true);
}

async function openAt(page: Page, query: string): Promise<void> {
  await page.goto(`/?map=synthetic&vph=0&${query}`);
  await waitForTiles(page);
  await page.mouse.click(CENTRE.x, CENTRE.y);
  await expect(page.locator(inspector)).toBeVisible({ timeout: 10_000 });
}

function body(page: Page) {
  return page.locator(`${inspector} .inspector-body`);
}

async function editRoad(page: Page): Promise<void> {
  await openAt(page, 'cx=975&cy=450&z=16');
  await expect(page.locator(inspector)).toContainText('Road 3');
  await expect(body(page)).toHaveAttribute('data-lanes-forward', '1');
  const plus = page.locator('.stepper', { hasText: 'Lanes →' }).getByRole('button', { name: '+' });
  await plus.hover();
  await expect(page.getByRole('tooltip')).toContainText('₱', { timeout: 5_000 });
  await plus.click();
  await expect(body(page)).toHaveAttribute('data-lanes-forward', '2', { timeout: 10_000 });
  expect(await spent(page)).toBeGreaterThan(0);
}

async function capture(page: Page, path: string): Promise<void> {
  await page.mouse.move(CORNER.x, CORNER.y);
  await page.screenshot({ path });
}

async function signalise(page: Page): Promise<void> {
  await openAt(page, 'cx=150&cy=750&z=10');
  await expect(body(page)).toHaveAttribute('data-control', 'Priority');
  const before = await numberAttr(page, debug, 'data-signal-pills');
  await page.getByRole('button', { name: 'Signal', exact: true }).click();
  await expect(body(page)).toHaveAttribute('data-control', 'Signal', { timeout: 10_000 });
  await expect
    .poll(async () => numberAttr(page, debug, 'data-signal-pills'), { timeout: 10_000 })
    .toBeGreaterThanOrEqual(before + 3);
}

async function banTurn(page: Page): Promise<void> {
  const before = await spent(page);
  await page.locator('.turn-toggle').first().click();
  await expect(body(page)).toHaveAttribute('data-turns-banned', '1', { timeout: 10_000 });
  await expect.poll(async () => spent(page), { timeout: 10_000 }).toBe(before + 20);
}

test('inspect and edit roads and junctions', async ({ page }, testInfo) => {
  test.setTimeout(90_000);
  const errors = collectErrors(page);
  await editRoad(page);
  await capture(page, testInfo.outputPath('road-panel.png'));
  await signalise(page);
  await banTurn(page);
  await capture(page, testInfo.outputPath('junction-panel.png'));
  await openAt(page, 'cx=750&cy=1500&z=6');
  const shadows = await numberAttr(page, debug, 'data-shadow-pieces');
  await page
    .getByRole('button', { name: /^Build flyover/ })
    .first()
    .click();
  await expect
    .poll(async () => numberAttr(page, debug, 'data-shadow-pieces'), { timeout: 5_000 })
    .toBeGreaterThan(shadows);
  await page.mouse.move(CORNER.x, CORNER.y);
  await page.screenshot({ path: testInfo.outputPath('flyover.png') });
  const afterFlyover = await spent(page);
  await page.locator('#undo').click();
  await expect.poll(async () => spent(page), { timeout: 10_000 }).toBeLessThan(afterFlyover);
  expect(errors).toEqual([]);
});
