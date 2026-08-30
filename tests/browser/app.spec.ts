import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('builds a bounded, redacted evidence preview', async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  await page.goto('/');
  await expect(page).toHaveTitle(/Alert Evidence Envelope/);
  await expect(page.locator('main')).toBeVisible();
  await expect(page.locator('h1')).toHaveCount(1);
  await page.getByRole('button', { name: 'Build safe preview' }).click();
  await expect(page.getByText('Envelope signed. No sample data was stored.')).toBeVisible();
  await expect(page.getByText('checkout-api', { exact: true })).toBeVisible();
  await page.getByText('Inspect signed JSON').click();
  await expect(page.locator('pre')).toContainText('[REDACTED]');
  expect(consoleErrors).toEqual([]);
});

test('keeps the expanded signed JSON keyboard-scrollable and free of serious accessibility violations', async ({ page }) => {
  for (const colorScheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme });
    await page.goto('/');
    await page.getByRole('button', { name: 'Build safe preview' }).click();
    await expect(page.getByText('Envelope signed. No sample data was stored.')).toBeVisible();
    await page.getByText('Inspect signed JSON').click();

    const signedJson = page.getByLabel('Signed evidence envelope JSON');
    await expect(signedJson).toHaveAttribute('tabindex', '0');
    const scrollBox = await signedJson.evaluate((element) => ({
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
    }));
    expect(scrollBox.scrollHeight).toBeGreaterThan(scrollBox.clientHeight);
    const pageWidth = await page.evaluate(() => ({
      document: document.documentElement.scrollWidth,
      viewport: window.innerWidth,
    }));
    expect(pageWidth.document).toBeLessThanOrEqual(pageWidth.viewport);
    await page.getByText('Inspect signed JSON').focus();
    await page.keyboard.press('Tab');
    await expect(signedJson).toBeFocused();
    await expect(signedJson).toHaveCSS('outline-style', 'solid');
    await page.keyboard.press('PageDown');
    await expect.poll(() => signedJson.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);

    await page.getByRole('link', { name: 'Skip to main content' }).focus();
    const results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
    expect(results.violations.filter((v) => ['serious', 'critical'].includes(v.impact || '')), colorScheme).toEqual([]);
  }
});

test('sends the transport-security policy on pages and APIs', async ({ request }) => {
  for (const path of ['/', '/health']) {
    const response = await request.get(path);
    expect(response.headers()['strict-transport-security'], path).toBe('max-age=63072000; includeSubDomains');
  }
});

test('exposes the skip link as the first keyboard target', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.press('Tab');
  const skipLink = page.getByRole('link', { name: 'Skip to main content' });
  await expect(skipLink).toBeFocused();
  await expect(skipLink).toHaveCSS('outline-style', 'solid');
});

test('legal routes are direct-linkable documents, including without JavaScript', async ({ page, request }) => {
  for (const path of ['/privacy', '/terms']) {
    const staticResponse = await request.get(path);
    expect(staticResponse.status(), `${path} must be useful without JavaScript`).toBe(200);
    expect(await staticResponse.text()).toContain(path === '/privacy' ? 'What the relay stores' : 'Operator responsibility');
    const response = await page.goto(path);
    expect(response?.status(), `${path} must be a direct-linkable legal document`).toBe(200);
    await expect(page.locator('h1')).toHaveCount(1);
    await expect(page.locator('main')).toBeVisible();
  }
});

test('reports the compiled immutable build identity', async ({ request }) => {
  const response = await request.get('/health');
  expect(response.status()).toBe(200);
  await expect(response.json()).resolves.toEqual({
    status: 'ok',
    build: process.env.PLAYWRIGHT_EXPECTED_BUILD_SHA,
  });
});

test('never widens the 390px mobile viewport', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  const widths = await page.evaluate(() => ({
    document: document.documentElement.scrollWidth,
    viewport: window.innerWidth,
    fieldKit: Math.ceil(document.querySelector<HTMLElement>('.field-kit')!.getBoundingClientRect().width),
  }));
  expect(widths.fieldKit).toBeLessThanOrEqual(widths.viewport);
  expect(widths.document).toBeLessThanOrEqual(widths.viewport);
});

test('keeps the current shell usable and reports offline state', async ({ page, context }) => {
  await page.goto('/');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByText('Browser offline')).toBeAttached();
  await expect(page.locator('h1')).toContainText('Send the evidence.');
  await context.setOffline(false);
});

test('keeps an updateable offline shell', async ({ page }) => {
  await page.goto('/');
  const shell = await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready;
    await registration.update();
    return (await caches.keys()).includes('envelope-shell-v2');
  });
  expect(shell).toBe(true);
});
