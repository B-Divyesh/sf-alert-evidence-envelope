import { defineConfig, devices } from '@playwright/test';

const testBuildSha = 'test-build-identity';
process.env.PLAYWRIGHT_EXPECTED_BUILD_SHA = testBuildSha;

export default defineConfig({
  testDir: './tests/browser',
  timeout: 30_000,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: 'line',
  use: { baseURL: 'http://127.0.0.1:4178', trace: 'retain-on-failure' },
  projects: [
    { name: 'desktop-chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile-chromium', use: { ...devices['iPhone 13'], browserName: 'chromium', viewport: { width: 390, height: 844 } } },
  ],
  webServer: {
    command: `PORT=4178 DATABASE_URL=sqlite:data/playwright.db?mode=rwc STATIC_DIR=dist ENVELOPE_SIGNING_KEY=playwright-signing-key-at-least-32-bytes ADMIN_TOKEN=test-admin-token-with-at-least-32-characters INBOUND_TOKEN=test-inbound-token-with-at-least-32-characters BUILD_SHA=${testBuildSha} cargo run --quiet`,
    url: 'http://127.0.0.1:4178/health',
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
