// Runs a throwaway script against the dev server. See AI notes/Browser harness.
// pnpm browse <script.mjs> [--firefox] [--headed] [--stub]
import { chromium, firefox } from "@playwright/test";
import path from "node:path";
import { pathToFileURL } from "node:url";
import * as hf from "../../e2e/helpers.ts";

const BASE_URL = "http://localhost:5173";
const args = process.argv.slice(2);
const flags = new Set(args.filter((a) => a.startsWith("--")));
const script = args.find((a) => !a.startsWith("--"));

if (!script) {
  console.error(
    "usage: pnpm browse <script.mjs> [--firefox] [--headed] [--stub]",
  );
  process.exit(1);
}
if (!(await fetch(BASE_URL).catch(() => null))) {
  console.error(`nothing on ${BASE_URL} - start it with pnpm dev`);
  process.exit(1);
}

const run = (await import(pathToFileURL(path.resolve(script)).href)).default;
const browser = await (flags.has("--firefox") ? firefox : chromium).launch({
  headless: !flags.has("--headed"),
});
const page = await browser.newPage({
  baseURL: BASE_URL,
  viewport: { width: 1600, height: 1000 },
});
const errors = hf.watch_errors(page);
if (flags.has("--stub")) await hf.stub_market(page);

let failed = false;
try {
  await run({ page, hf });
} catch (e) {
  failed = true;
  console.error(e);
}
console.log("page errors:", errors);
await browser.close();
process.exit(failed ? 1 : 0);
