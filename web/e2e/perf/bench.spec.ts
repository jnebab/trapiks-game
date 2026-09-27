import { appendFileSync, mkdirSync } from 'node:fs';
import { expect, test, type Page } from '@playwright/test';

const OUT_DIR = 'test-results';

function report(label: string, values: object): void {
  mkdirSync(OUT_DIR, { recursive: true });
  appendFileSync(`${OUT_DIR}/perf.jsonl`, `${JSON.stringify({ label, ...values })}\n`);
}

const debug = '#debug';
const MEASURE_MS = 60_000;
const WARMUP_TIMEOUT_MS = 20 * 60_000;

async function attr(page: Page, name: string): Promise<number> {
  return Number(await page.locator(debug).getAttribute(name));
}

async function waitForTick(page: Page, tick: number): Promise<void> {
  await expect
    .poll(async () => attr(page, 'data-tick'), { timeout: WARMUP_TIMEOUT_MS, intervals: [2_000] })
    .toBeGreaterThanOrEqual(tick);
}

async function waitForTiles(page: Page): Promise<void> {
  await expect
    .poll(async () => (await attr(page, 'data-first-tiles-ms')) > 0, { timeout: 60_000 })
    .toBe(true);
}

async function stepSummary(page: Page): Promise<Record<string, number>> {
  const names = ['step-count', 'step-p50', 'step-p95', 'step-max', 'vehicles'];
  const values = await Promise.all(names.map(async (name) => attr(page, `data-${name}`)));
  return Object.fromEntries(names.map((name, i) => [name, values[i] ?? 0]));
}

async function frameSummary(page: Page): Promise<Record<string, number>> {
  return {
    p50: await attr(page, 'data-frame-p50'),
    p95: await attr(page, 'data-frame-p95'),
  };
}

async function measureSteps(page: Page, url: string, warmup: number): Promise<void> {
  await page.goto(url);
  await waitForTiles(page);
  await page.getByRole('button', { name: 'Max', exact: true }).click();
  await waitForTick(page, warmup);
  await page.getByRole('button', { name: '1×', exact: true }).click();
  await page.waitForTimeout(MEASURE_MS);
  report(`steps ${url}`, await stepSummary(page));
  report(`frames ${url}`, await frameSummary(page));
}

test.describe.configure({ mode: 'serial', timeout: 30 * 60_000 });

test('wasm city step on grid120 at street zoom', async ({ page }) => {
  const url = '/?map=grid120&vph=100000&perf=1&perfFrom=9000&cx=9000&cy=9000&z=10#/sandbox';
  await measureSteps(page, url, 9_000);
});

test('wasm region step on grid120', async ({ page }) => {
  const url = '/?map=grid120&region=9000,9000,2000&vph=36000&perf=1&perfFrom=3000#/sandbox';
  await measureSteps(page, url, 3_000);
});

test('city zoom frames over grid120', async ({ page }) => {
  await page.goto('/?map=grid120&vph=0&perf=1&cx=9000&cy=9000&z=0.1#/sandbox');
  await waitForTiles(page);
  await page.waitForTimeout(15_000);
  report('frames city zoom', await frameSummary(page));
});

for (const map of ['synthetic', 'grid120']) {
  test(`load time ${map}`, async ({ page }) => {
    await page.goto(`/?map=${map}&vph=0#/sandbox`);
    await waitForTiles(page);
    const ready = await attr(page, 'data-ready-ms');
    const tiles = await attr(page, 'data-first-tiles-ms');
    report(`load ${map}`, { ready, tiles });
  });
}
