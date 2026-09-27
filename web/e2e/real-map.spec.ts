import { appendFileSync, existsSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { expect, test, type Page } from '@playwright/test';
import { projectLatLon, readMapOrigin, type MapPoint } from './map-origin';

const MAP_FILE = fileURLToPath(new URL('../public/maps/metro-manila.bin.gz', import.meta.url));
const HAS_MAP = existsSync(MAP_FILE);
const SHOTS = process.env.REAL_MAP_SHOTS;
const OUT_DIR = 'test-results';
const debug = '#debug';
const CORNER = { x: 20, y: 600 } as const;
const ORTIGAS = { lat: 14.591701, lon: 121.058356 } as const;
const SKYWAY_RAMP = { lat: 14.527608, lon: 121.02392 } as const;
const STREET_ZOOM = 6;
const RAMP_ZOOM = 2.5;
const TRAFFIC_VPH = 20_000;
const MIN_TRAFFIC = 40;
const WARM_TICKS = 1_500;

test.skip(!HAS_MAP, 'web/public/maps/metro-manila.bin.gz is not built yet');
test.describe.configure({ timeout: 180_000 });

function located(point: { lat: number; lon: number }): MapPoint {
  return projectLatLon(readMapOrigin(MAP_FILE), point.lat, point.lon);
}

function cameraQuery(point: MapPoint, zoom: number): string {
  return `cx=${point.x.toFixed(0)}&cy=${point.y.toFixed(0)}&z=${String(zoom)}`;
}

function report(values: { ready: number; tiles: number }): void {
  mkdirSync(OUT_DIR, { recursive: true });
  appendFileSync(
    `${OUT_DIR}/perf.jsonl`,
    `${JSON.stringify({ label: 'load metro-manila', ...values })}\n`,
  );
  test.info().annotations.push({ type: 'load', description: JSON.stringify(values) });
}

async function numberAttr(page: Page, name: string): Promise<number> {
  return Number(await page.locator(debug).getAttribute(name));
}

async function waitForTiles(page: Page): Promise<void> {
  await expect
    .poll(
      async () => {
        const built = await numberAttr(page, 'data-tiles-built');
        const visible = await numberAttr(page, 'data-tiles-visible');
        return built === visible && visible > 0;
      },
      { timeout: 120_000, intervals: [1_000] },
    )
    .toBe(true);
}

async function warmTraffic(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Max', exact: true }).click();
  await expect
    .poll(async () => numberAttr(page, 'data-tick'), { timeout: 100_000, intervals: [2_000] })
    .toBeGreaterThanOrEqual(WARM_TICKS);
  await page.getByRole('button', { name: '1×', exact: true }).click();
  expect(await numberAttr(page, 'data-vehicles')).toBeGreaterThan(MIN_TRAFFIC);
}

async function shot(page: Page, name: string, outputPath: (name: string) => string) {
  await page.mouse.move(CORNER.x, CORNER.y);
  const path = SHOTS === undefined ? outputPath(name) : `${SHOTS}/${name}`;
  await page.screenshot({ path });
}

async function measureBaseline(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Challenges' }).click();
  await page.locator('.challenge-card', { hasText: 'EDSA–Ortigas' }).click();
  const objective = page.locator('#objective');
  await expect
    .poll(async () => objective.textContent(), { timeout: 150_000, intervals: [2_000] })
    .toContain('baseline ');
  await expect(objective).not.toContainText('Measuring');
}

test('load time on the real map', async ({ page }) => {
  test.setTimeout(180_000);
  await page.goto('/?map=metro-manila&vph=0&perf=1#/sandbox');
  await expect
    .poll(async () => (await numberAttr(page, 'data-first-tiles-ms')) > 0, { timeout: 150_000 })
    .toBe(true);
  const ready = await numberAttr(page, 'data-ready-ms');
  const tiles = await numberAttr(page, 'data-first-tiles-ms');
  report({ ready, tiles });
});

test('title, EDSA–Ortigas baseline and street zoom', async ({ page }, testInfo) => {
  test.setTimeout(180_000);
  const out = (name: string): string => testInfo.outputPath(name);
  await page.goto(`/?map=metro-manila&${cameraQuery(located(ORTIGAS), STREET_ZOOM)}#/`);
  await expect(page.locator('#title')).toBeVisible({ timeout: 60_000 });
  await expect(page.locator('.attribution').first()).toContainText('OpenStreetMap');
  await measureBaseline(page);
  await waitForTiles(page);
  await warmTraffic(page);
  await shot(page, 'real-edsa-ortigas-street.png', out);
});

test('city zoom over Metro Manila', async ({ page }, testInfo) => {
  test.setTimeout(180_000);
  await page.goto('/?map=metro-manila&vph=0#/sandbox');
  await waitForTiles(page);
  await shot(page, 'real-city.png', (name) => testInfo.outputPath(name));
});

test('skyway ramp near Magallanes', async ({ page }, testInfo) => {
  test.setTimeout(180_000);
  const query = cameraQuery(located(SKYWAY_RAMP), RAMP_ZOOM);
  await page.goto(`/?map=metro-manila&vph=${String(TRAFFIC_VPH)}&${query}#/sandbox`);
  await waitForTiles(page);
  await warmTraffic(page);
  await shot(page, 'real-skyway-ramp.png', (name) => testInfo.outputPath(name));
});
