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

test('has no serious accessibility violations', async ({ page }) => {
  await page.goto('/');
  for (const colorScheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme });
    const results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
    expect(results.violations.filter((v) => ['serious', 'critical'].includes(v.impact || '')), colorScheme).toEqual([]);
  }
});

test('legal routes have one heading and a main landmark', async ({ page }) => {
  for (const path of ['/privacy', '/terms']) {
    await page.goto(path);
    await expect(page.locator('h1')).toHaveCount(1);
    await expect(page.locator('main')).toBeVisible();
  }
});
