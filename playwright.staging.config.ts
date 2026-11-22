// domain_move.spec.ts against the deployed staging sites (pnpm test:e2e:staging); deploy pnpm staging-build first
import { defineConfig } from "@playwright/test";
import base from "./playwright.config";

process.env.HF_MOVE_OLD = "https://migration2.honing-forecast.pages.dev";
process.env.HF_MOVE_NEW = "https://test.honing-forecast.com";

export default defineConfig({
  ...base,
  testMatch: "domain_move.spec.ts",
  testIgnore: [],
  webServer: undefined,
});
