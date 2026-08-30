import { expect, test, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const seriousAxe = async (page: Page) => {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
    .analyze();
  return results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact || ''));
};

test('@claim:demo-envelope opens one-click sample and builds a safe envelope', async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  await page.goto('/');
  await expect(page.locator('h1')).toHaveText('Send safe evidence with every alert');
  await page.getByRole('link', { name: 'Try it with sample data' }).click();
  await expect(page).toHaveURL(/\/demo$/);
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', 'https://alert-evidence-envelope.sociobot.in/demo');
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
  await expect(page.getByText('checkout-api', { exact: true })).toBeVisible();
  await expect(page.getByText('payment authorization timed out', { exact: true })).toBeVisible();
  await expect(page.getByText('8/27/2026', { exact: false })).toBeVisible();
  await page.getByText('Inspect signed JSON').click();
  const json = page.getByLabel('Signed evidence envelope JSON');
  await expect(json).toContainText('[REDACTED]');
  await expect(json).toContainText(/hmac-sha256=[a-f0-9]{64}/);
  expect(consoleErrors).toEqual([]);
});

test('keeps expanded JSON keyboard-scrollable and accessible in both themes', async ({ page }) => {
  for (const colorScheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme });
    await page.goto('/demo');
    await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
    await page.getByText('Inspect signed JSON').click();
    const signedJson = page.getByLabel('Signed evidence envelope JSON');
    await expect(signedJson).toHaveAttribute('tabindex', '0');
    const scrollBox = await signedJson.evaluate((element) => ({
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
    }));
    expect(scrollBox.scrollHeight).toBeGreaterThan(scrollBox.clientHeight);
    await page.getByText('Inspect signed JSON').focus();
    await page.keyboard.press('Tab');
    await expect(signedJson).toBeFocused();
    await expect(signedJson).toHaveCSS('outline-style', 'solid');
    await page.keyboard.press('PageDown');
    await expect.poll(() => signedJson.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);
    expect(await seriousAxe(page), colorScheme).toEqual([]);
  }
});

test('protects every real route before body parsing', async ({ request }) => {
  const checks: Array<{ method: 'get' | 'post' | 'put'; path: string }> = [
    { method: 'get', path: '/api/v1/config' },
    { method: 'get', path: '/api/v1/history' },
    { method: 'put', path: '/api/v1/config' },
    { method: 'post', path: '/api/v1/preview' },
    { method: 'post', path: '/api/v1/relay/primary' },
  ];
  for (const check of checks) {
    const response = await request[check.method](check.path, {
      headers: { 'content-type': 'application/json' },
      data: '{',
    });
    expect(response.status(), `${check.method.toUpperCase()} ${check.path}`).toBe(401);
  }
});

test('sends security and cache policy headers', async ({ request }) => {
  for (const path of ['/', '/privacy', '/terms', '/health', '/api/v1/config']) {
    const response = await request.get(path);
    const headers = response.headers();
    expect(headers['strict-transport-security'], path).toBe('max-age=63072000; includeSubDomains');
    expect(headers['x-content-type-options'], path).toBe('nosniff');
    expect(headers['x-frame-options'], path).toBe('DENY');
    expect(headers['content-security-policy'], path).toContain("frame-ancestors 'none'");
    expect(headers['cache-control'], path.startsWith('/api/') || path === '/health' ? path : undefined)
      .toBe(path.startsWith('/api/') || path === '/health' ? 'no-store' : 'no-cache');
  }
});

test('exposes the skip link as the first keyboard target', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.press('Tab');
  const skipLink = page.getByRole('link', { name: 'Skip to main content' });
  await expect(skipLink).toBeFocused();
  await expect(skipLink).toHaveCSS('outline-style', 'solid');
});

test('legal routes pass light and dark accessibility audits at 390px', async ({ page, request }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const path of ['/privacy', '/terms']) {
    const staticResponse = await request.get(path);
    expect(staticResponse.status()).toBe(200);
    expect(await staticResponse.text()).toContain(path === '/privacy' ? 'What the relay stores' : 'Operator responsibility');
    for (const colorScheme of ['light', 'dark'] as const) {
      await page.emulateMedia({ colorScheme });
      await page.goto(path);
      await expect(page.locator('h1')).toHaveCount(1);
      await expect(page.locator('main')).toBeVisible();
      expect(await seriousAxe(page), `${path} ${colorScheme}`).toEqual([]);
    }
  }
});

test('reports the compiled immutable build identity in health and the footer', async ({ page, request }) => {
  const response = await request.get('/health');
  expect(response.status()).toBe(200);
  await expect(response.json()).resolves.toEqual({
    status: 'ok',
    build: process.env.PLAYWRIGHT_EXPECTED_BUILD_SHA,
  });
  await page.goto('/');
  await expect(page.locator('.provenance')).toContainText(`Build ${process.env.PLAYWRIGHT_EXPECTED_BUILD_SHA?.slice(0, 12)}`);
  await page.goto('/privacy');
  await expect(page.locator('[data-build]')).toContainText(`Build ${process.env.PLAYWRIGHT_EXPECTED_BUILD_SHA?.slice(0, 12)}`);
});

test('never widens the 390px mobile viewport', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const path of ['/', '/demo', '/privacy', '/terms']) {
    await page.goto(path);
    if (path === '/demo') {
      await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
      await page.getByText('Inspect signed JSON').click();
    }
    const widths = await page.evaluate(() => ({
      document: document.documentElement.scrollWidth,
      viewport: window.innerWidth,
    }));
    expect(widths.document, path).toBeLessThanOrEqual(widths.viewport);
  }
});

test('@claim:offline-demo reloads the sample offline after the first visit', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await page.goto(`${baseURL}/demo`);
  await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
  await page.evaluate(() => navigator.serviceWorker.ready);
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await expect(page.getByText('Offline sample ready. Demo data was not stored.')).toBeVisible();
  await expect(page.getByText('checkout-api', { exact: true })).toBeVisible();
  await context.setOffline(false);
  await context.close();
});

test('keeps an updateable offline shell', async ({ page }) => {
  await page.goto('/');
  const shell = await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready;
    await registration.update();
    return (await caches.keys()).includes('envelope-shell-v3');
  });
  expect(shell).toBe(true);
});

test('@claim:no-tracking keeps the sample flow on the product origin', async ({ page, baseURL }) => {
  const origins = new Set<string>();
  page.on('request', (request) => origins.add(new URL(request.url()).origin));
  await page.goto('/demo');
  await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
  expect([...origins]).toEqual([new URL(baseURL!).origin]);
});

test('resets and exits the isolated demo without retaining its namespace', async ({ page }) => {
  await page.goto('/demo');
  await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
  const firstSession = await page.evaluate(() => localStorage.getItem('demo:alert-evidence-envelope:session'));
  await page.getByText('Inspect signed JSON').click();
  const firstEnvelope = JSON.parse(await page.getByLabel('Signed evidence envelope JSON').textContent() || '{}').id;
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect.poll(() => page.evaluate(() => localStorage.getItem('demo:alert-evidence-envelope:session'))).not.toBe(firstSession);
  await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
  await page.getByText('Inspect signed JSON').click();
  const secondEnvelope = JSON.parse(await page.getByLabel('Signed evidence envelope JSON').textContent() || '{}').id;
  expect(secondEnvelope).not.toBe(firstEnvelope);
  await page.getByRole('link', { name: 'Start for real' }).first().click();
  await expect(page).toHaveURL('/');
  expect(await page.evaluate(() => Object.keys(localStorage).filter((key) => key.startsWith('demo:')))).toEqual([]);
});

test('keeps content and primary actions available at 200% text size', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const path of ['/', '/demo', '/privacy', '/terms']) {
    await page.goto(path);
    await page.locator('html').evaluate((element) => { element.style.fontSize = '32px'; });
    const widths = await page.evaluate(() => ({ document: document.documentElement.scrollWidth, viewport: innerWidth }));
    expect(widths.document, path).toBeLessThanOrEqual(widths.viewport);
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('main')).toBeVisible();
  }
});

test('serves discovery metadata, icons, and a designed 404', async ({ page, request }) => {
  for (const path of ['/robots.txt', '/sitemap.xml', '/apple-touch-icon.png', '/assets/social-card.jpg']) {
    expect((await request.get(path)).status(), path).toBe(200);
  }
  const home = await request.get('/');
  const html = await home.text();
  expect(html).toContain('rel="canonical"');
  expect(html).toContain('property="og:image"');
  expect(html).toContain('name="twitter:card"');
  const response = await page.goto('/not-a-real-route');
  expect(response?.status()).toBe(404);
  await expect(page).toHaveTitle('Page not found — Alert Evidence Envelope');
  await expect(page.locator('h1')).toHaveText('We could not find this page');
});

test('keeps navigation touch targets at least 44px high', async ({ page }) => {
  for (const viewport of [{ width: 1440, height: 900 }, { width: 390, height: 844 }]) {
    await page.setViewportSize(viewport);
    await page.goto('/');
    const heights = await page.locator('.brand, .site-header nav a, .footer-links a').evaluateAll((elements) =>
      elements.map((element) => ({ text: element.textContent?.trim(), height: element.getBoundingClientRect().height }))
        .filter((element) => element.height > 0),
    );
    for (const target of heights) expect(target.height, `${viewport.width}px ${target.text}`).toBeGreaterThanOrEqual(44);
  }
});

test('@claim:rate-limit limits each forwarded client and returns a useful retry delay', async ({ request }, testInfo) => {
  const octet = testInfo.project.name.startsWith('mobile') ? '44' : '43';
  const responses = await Promise.all(Array.from({ length: 60 }, () => request.get('/api/v1/config', {
    headers: { 'x-forwarded-for': `198.51.100.${octet}` },
  })));
  const limited = responses.filter((response) => response.status() === 429);
  expect(limited.length).toBeGreaterThan(0);
  expect(limited[0].headers()['retry-after']).toBe('1');
  const otherClient = await request.get('/api/v1/config', { headers: { 'x-forwarded-for': `203.0.113.${octet}` } });
  expect(otherClient.status()).toBe(401);
});

test('@claim:field-kit-purchase exposes a working one-time checkout', async ({ page, request }) => {
  await page.goto('/');
  const checkout = page.getByRole('link', { name: 'Buy the Field Kit' });
  const productionUrl = 'https://api.sociobot.in/api/v1/products/alert-evidence-envelope/checkout';
  await expect(checkout).toHaveAttribute('href', productionUrl);
  await expect(page.getByText('Self-hosted core is free; Field Kit costs $39 once', { exact: true })).toBeVisible();
  const production = await request.get(productionUrl, { maxRedirects: 0 });
  expect([302, 303, 307]).toContain(production.status());
  expect(production.headers().location).toMatch(/^https:\/\/checkout\.dodopayments\.com\//);
  const pilot = await request.get(productionUrl.replace('https://api.sociobot.in', 'https://pilot-api.sociobot.in'), { maxRedirects: 0 });
  expect([302, 303, 307]).toContain(pilot.status());
  expect(pilot.headers().location).toMatch(/^https:\/\/test\.checkout\.dodopayments\.com\//);
});

test('restores a Field Kit license and strips returned tokens from the URL', async ({ page }) => {
  await page.route('https://api.sociobot.in/api/v1/products/alert-evidence-envelope/verify**', async (route) => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok' }) });
  });
  await page.goto('/?license=returned-test-license');
  await expect(page).toHaveURL('/');
  await expect(page.getByText('Field Kit unlocked')).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('sb_license:alert-evidence-envelope'))).toBe('returned-test-license');
});
