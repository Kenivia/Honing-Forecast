import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "tests/e2e",
  outputDir: "tests/test-results",
  fullyParallel: true,
  // more workers than this overload the dev server and tests time out; each worker is a browser, so memory-bound
  workers: 4,
  use: {
    baseURL: "http://localhost:5173",
    trace: "retain-on-failure",
  },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
  ],
  // pnpm dev rebuilds wasm first, hence the long timeout
  webServer: {
    command: "pnpm dev",
    url: "http://localhost:5173",
    reuseExistingServer: true,
    timeout: 300_000,
  },
});
