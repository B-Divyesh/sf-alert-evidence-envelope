import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/browser',
  timeout: 30_000,
  retries: 0,
  workers: 1,
  reporter: 'line',
  use: { baseURL: 'http://127.0.0.1:4178', trace: 'retain-on-failure' },
  projects: [
    { name: 'desktop-chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile-chromium', use: { ...devices['iPhone 13'], browserName: 'chromium', viewport: { width: 390, height: 844 } } },
  ],
  webServer: {
    command: 'PORT=4178 DATABASE_URL=sqlite:data/playwright.db?mode=rwc STATIC_DIR=dist ENVELOPE_SIGNING_KEY=playwright-secret BUILD_SHA=5e9f77e56c4f28e6b1d848d3de611091bce8bb83 cargo run --quiet',
    url: 'http://127.0.0.1:4178/health',
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
