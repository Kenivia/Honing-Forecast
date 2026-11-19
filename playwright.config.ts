import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "tests/e2e",
  outputDir: "tests/test-results",
  fullyParallel: true,
  use: {
    baseURL: "http://localhost:5173",
    trace: "retain-on-failure",
  },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
  ],
  // pnpm dev rebuilds wasm first, hence the long timeout
  webServer: [
    {
      command: "pnpm dev",
      url: "http://localhost:5173",
      reuseExistingServer: true,
      timeout: 300_000,
    },
    // domain_move.spec.ts: localhost and 127.0.0.1 are the old and new sites; uses the wasm pnpm dev built
    {
      command: "npx vite --port 5174 --strictPort --host ::",
      url: "http://127.0.0.1:5174",
      reuseExistingServer: true,
      timeout: 300_000,
      env: {
        VITE_MIGRATE_FROM: "http://localhost:5174",
        VITE_MIGRATE_TO: "http://127.0.0.1:5174",
      },
    },
  ],
});
