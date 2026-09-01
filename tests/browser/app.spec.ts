import { expect, test, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { createHmac } from 'node:crypto';
import { readFileSync } from 'node:fs';

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
  const envelope = JSON.parse(await json.textContent() || '{}');
  const signature = envelope.signature;
  envelope.signature = '';
  expect(signature).toBe(`hmac-sha256=${createHmac('sha256', 'playwright-signing-key-at-least-32-bytes')
    .update(JSON.stringify(envelope)).digest('hex')}`);
  expect(envelope.evidence_items).toBeLessThanOrEqual(20);
  expect(envelope.evidence_bytes).toBeLessThanOrEqual(32_768);
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
  for (const path of ['/fonts/inter-latin.woff2', '/assets/evidence-terrain-960.webp', '/assets/social-card.jpg', '/favicon.svg']) {
    expect((await request.get(path)).headers()['cache-control'], path).toBe('no-cache');
  }
  const html = await (await request.get('/')).text();
  const versionedAssets = [...html.matchAll(/(?:src|href)="(\/assets\/index-[^"]+\.(?:js|css))"/g)]
    .map((match) => match[1]);
  expect(versionedAssets).toHaveLength(2);
  for (const path of versionedAssets) {
    expect((await request.get(path)).headers()['cache-control'], path)
      .toBe('public, max-age=31536000, immutable');
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

test('discovers the mobile LCP image before app boot without loading webfonts', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const fontRequests: string[] = [];
  const requestOrder: string[] = [];
  page.on('request', (request) => {
    requestOrder.push(new URL(request.url()).pathname);
    if (request.resourceType() === 'font') fontRequests.push(new URL(request.url()).pathname);
  });
  await page.goto('/');
  await expect(page.locator('h1')).toHaveText('Send safe evidence with every alert');
  await expect(page.getByRole('link', { name: 'Try it with sample data' })).toBeVisible();
  expect(fontRequests).toEqual([]);
  expect(requestOrder.indexOf('/assets/evidence-terrain-960.webp')).toBeGreaterThanOrEqual(0);
  expect(requestOrder.indexOf('/assets/evidence-terrain-960.webp'))
    .toBeLessThan(requestOrder.findIndex((path) => path.startsWith('/assets/index-') && path.endsWith('.js')));
  expect(await seriousAxe(page)).toEqual([]);
});

test('@claim:offline-demo reloads the sample offline after the first visit', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await page.goto(`${baseURL}/demo`);
  await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
  await page.evaluate(() => navigator.serviceWorker.ready);
  const offlineRequests: string[] = [];
  const failedRequests: string[] = [];
  let offline = false;
  page.on('request', (request) => { if (offline) offlineRequests.push(request.url()); });
  page.on('requestfailed', (request) => {
    if (offline) failedRequests.push(`${request.url()}: ${request.failure()?.errorText}`);
  });
  await context.setOffline(true);
  offline = true;
  await page.reload();
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await expect(page.getByText('Offline sample ready. Demo data was not stored.')).toBeVisible();
  await expect(page.getByText('checkout-api', { exact: true })).toBeVisible();
  expect(offlineRequests.filter((url) => /\/(?:health|api\/)/.test(new URL(url).pathname))).toEqual([]);
  expect(failedRequests).toEqual([]);
  await context.setOffline(false);
  await context.close();
});

test('keeps an updateable offline shell', async ({ page }) => {
  await page.goto('/');
  const shell = await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready;
    await registration.update();
    return (await caches.keys()).includes('envelope-shell-v5');
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
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', 'https://alert-evidence-envelope.sociobot.in/404');
  await expect(page.locator('meta[property="og:url"]')).toHaveAttribute('content', 'https://alert-evidence-envelope.sociobot.in/404');
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute('content', '#f3f0e5');
});

test('moves focus and announces the new route after in-app navigation', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('link', { name: 'Demo' }).click();
  await expect(page).toHaveURL(/\/demo$/);
  await expect(page.locator('h1')).toBeFocused();
  await expect(page.locator('[aria-live="polite"]').first()).toContainText('Inspect a sample evidence envelope');
  await page.goBack();
  await expect(page.locator('h1')).toBeFocused();
});

test('keeps every 390px interactive target at least 44 by 44px', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const path of ['/', '/demo', '/privacy', '/terms']) {
    await page.goto(path);
    if (path === '/demo') await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
    const targets = await page.locator('a[href], button, input:not([type="hidden"]), select, textarea, summary').evaluateAll((elements) =>
      elements.map((element) => {
        const box = element.getBoundingClientRect();
        return {
          name: element.getAttribute('aria-label') || element.textContent?.trim() || element.getAttribute('name') || element.tagName,
          width: box.width,
          height: box.height,
        };
      }).filter((element) => element.width > 0 && element.height > 0),
    );
    for (const target of targets) {
      expect(target.width, `${path} ${target.name} width`).toBeGreaterThanOrEqual(44);
      expect(target.height, `${path} ${target.name} height`).toBeGreaterThanOrEqual(44);
    }
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

test('@claim:field-kit-purchase shows the price and official checkout action', async ({ page }) => {
  await page.goto('/');
  const checkout = page.getByRole('link', { name: 'Buy the Field Kit' });
  const productionUrl = 'https://api.sociobot.in/api/v1/products/alert-evidence-envelope/checkout';
  await expect(checkout).toHaveAttribute('href', productionUrl);
  await expect(page.getByText('Self-hosted core is free; Field Kit costs $39 once', { exact: true })).toBeVisible();
  expect(new URL(await checkout.getAttribute('href') || '').origin).toBe('https://api.sociobot.in');
});

test('@claim:local-policy-presets keeps named redaction presets on this device', async ({ page }) => {
  const origins = new Set<string>();
  page.on('request', (request) => origins.add(new URL(request.url()).origin));
  await page.addInitScript(() => {
    localStorage.setItem('sb_license:alert-evidence-envelope', 'fixture-license');
    localStorage.setItem(
      'sb_license:alert-evidence-envelope:verdict',
      JSON.stringify({ valid: true, checkedAt: Date.now() }),
    );
  });
  await page.goto('/');
  await expect(page.getByText('Field Kit unlocked')).toBeVisible();
  await page.getByLabel('Redact keys comma-separated').fill('email, token, customer_id');
  await page.getByLabel('Preset name').fill('Customer Slack');
  await page.getByRole('button', { name: 'Save current redaction policy' }).click();
  await expect(page.getByRole('button', { name: /Customer Slack/ })).toBeVisible();
  expect(await page.evaluate(() => JSON.parse(localStorage.getItem('envelope:presets') || '[]')))
    .toEqual([{ name: 'Customer Slack', fields: ['email', 'token', 'customer_id'] }]);

  await page.reload();
  const preset = page.getByRole('button', { name: /Customer Slack/ });
  await expect(preset).toBeVisible();
  await page.getByLabel('Redact keys comma-separated').fill('password');
  await preset.click();
  await expect(page.getByLabel('Redact keys comma-separated')).toHaveValue('email, token, customer_id');
  expect([...origins]).toEqual([new URL(page.url()).origin]);
});

test('restores a Field Kit license and strips returned tokens from the URL', async ({ page }) => {
  await page.route('https://api.sociobot.in/api/v1/products/alert-evidence-envelope/verify', async (route) => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok' }) });
  });
  await page.goto('/?license=returned-test-license');
  await expect(page).toHaveURL('/');
  await expect(page.getByText('Field Kit unlocked')).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('sb_license:alert-evidence-envelope'))).toBe('returned-test-license');
});

test('@claim:mobile-demo-result shows the transformed envelope above the fold', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/demo');
  await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
  for (const text of ['checkout-api', 'payment authorization timed out', '[REDACTED]', 'Envelope signed. Demo data was not stored.']) {
    const box = await page.getByText(text, { exact: text === 'checkout-api' || text === 'payment authorization timed out' }).first().boundingBox();
    expect(box?.y, text).toBeLessThan(844);
  }
});

test('@claim:license-transport uses an authorization header and never a token URL', async ({ page }) => {
  let seenUrl = ''; let authorization = '';
  await page.route('https://api.sociobot.in/api/v1/products/alert-evidence-envelope/verify', async (route) => {
    seenUrl = route.request().url(); authorization = route.request().headers().authorization || '';
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: false, reason: 'invalid' }) });
  });
  await page.goto('/?license=fixture-license-token');
  await expect(page.getByText('License no longer active')).toBeVisible();
  expect(seenUrl).not.toContain('license=');
  expect(authorization).toBe('Bearer fixture-license-token');
});

test('@claim:license-throttle waits after a failed verification attempt', async ({ page }) => {
  let attempts = 0;
  await page.route('https://api.sociobot.in/api/v1/products/alert-evidence-envelope/verify', async (route) => { attempts += 1; await route.abort(); });
  await page.addInitScript(() => localStorage.setItem('sb_license:alert-evidence-envelope', 'fixture-license'));
  await page.goto('/'); await page.reload();
  expect(attempts).toBe(1);
});

test('@claim:free-core keeps the preview and export action free', async ({ page }) => {
  const billingRequests: string[] = [];
  page.on('request', (request) => { if (request.url().includes('api.sociobot.in')) billingRequests.push(request.url()); });
  await page.goto('/demo');
  await expect(page.getByText('Envelope signed. Demo data was not stored.')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Copy envelope JSON' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Buy the Field Kit' })).not.toBeVisible();
  expect(billingRequests).toEqual([]);
});

test('@claim:license-revocation hides Field Kit controls but leaves free preview', async ({ page }) => {
  await page.route('https://api.sociobot.in/api/v1/products/alert-evidence-envelope/verify', async (route) => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: false, reason: 'revoked' }) });
  });
  await page.goto('/?license=revoked-fixture');
  await expect(page.getByText('License no longer active')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Save current redaction policy' })).not.toBeVisible();
  await expect(page.getByRole('button', { name: 'Build signed preview' })).toBeVisible();
});

test('@claim:provenance-license renders reviewed provenance', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('.provenance')).toContainText('generated for this product on 2026-08-27');
  const license = readFileSync('LICENSE', 'utf8');
  expect(license).toContain('MIT License');
  const metadata = readFileSync('assets/src/evidence-terrain.json', 'utf8');
  expect(metadata).toContain('2026-08-27');
});
