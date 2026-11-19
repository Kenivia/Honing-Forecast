import { expect, test } from "@playwright/test";
import fs from "node:fs";
import * as hf from "./helpers.ts";
import { roster_names } from "./bible_helpers.ts";

// Against the real lostark.bible API with the author's token: BIBLE_TOKEN in .env.local (gitignored).
// Get one by signing in on localhost:5173, then `JSON.parse(localStorage.bible_auth).token` in devtools.
// Read only: revoke is always stubbed, so signing out here never kills the token.
const ENV = new URL("../../.env.local", import.meta.url);
const TOKEN = fs.existsSync(ENV)
  ? fs
      .readFileSync(ENV, "utf8")
      .match(/^BIBLE_TOKEN=(.+)$/m)?.[1]
      .trim()
  : undefined;

test.skip(!TOKEN, "needs BIBLE_TOKEN in .env.local");

let errors: string[];
test.beforeEach(async ({ page }) => {
  errors = hf.watch_errors(page);
  await hf.stub_market(page);
  await page.route(/lostark\.bible\/oauth\/revoke/, (route) =>
    route.fulfill({
      status: 200,
      headers: { "access-control-allow-origin": "*" },
    }),
  );
  await page.addInitScript((token) => {
    if (sessionStorage.getItem("test_signed_in")) return;
    sessionStorage.setItem("test_signed_in", "1");
    localStorage.setItem(
      "bible_auth",
      JSON.stringify({
        token,
        expires_at: Date.now() + 86_400_000,
        name: "Live",
      }),
    );
  }, TOKEN);
});
test.afterEach(() => expect(errors).toEqual([]));

test("sync and refresh against the real API", async ({ page }) => {
  test.setTimeout(180_000);
  const api: string[] = [];
  page.on("response", (r) => {
    if (r.url().includes("lostark.bible/api/oauth"))
      api.push(`${r.status()} ${new URL(r.url()).pathname}`);
  });
  await hf.open(page, "/roster-setup");
  await expect(page.getByRole("button", { name: "Sign out" })).toBeVisible();
  const sync = page.getByRole("button", { name: "Sync with lostark.bible" });
  await sync.click();
  await expect(sync).toBeEnabled({ timeout: 120_000 });
  // a 401 would have signed us out
  await expect(page.getByRole("button", { name: "Sign out" })).toBeVisible();
  expect(api.filter((line) => !line.startsWith("200"))).toEqual([]);

  const names = (await roster_names(page)).flat();
  expect(names.length).toBeGreaterThan(0);
  expect(names.some((n) => n.startsWith("Newchar"))).toBe(false);
  await expect(
    page.getByText("Your roster on lostark.bible").first(),
  ).toBeVisible();

  // a linked character is locked and refreshes through the API
  const first = names[0];
  await hf.open(page, `/chars/${encodeURIComponent(first)}/calc`);
  await expect(page.getByLabel("Char name").first()).toHaveAttribute(
    "readonly",
    "",
  );
  await page
    .getByRole("button", { name: "Refresh from lostark.bible" })
    .first()
    .click();
  await expect(page.getByText(/Last fetched (?!never)/).first()).toBeVisible({
    timeout: 60_000,
  });
  expect(api.some((line) => line.startsWith("200 /api/oauth/loadouts/"))).toBe(
    true,
  );
  expect(api.filter((line) => !line.startsWith("200"))).toEqual([]);
});
