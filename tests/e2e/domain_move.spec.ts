// The domain move (index.html sender, migrate/index.html receiver, RosterConfig init), on the dev server:
// [::1]:5173 is the old site and localhost:5173 the new one (.env.development).
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import fs from "node:fs";
import crypto from "node:crypto";
import LZString from "lz-string";
import * as hf from "./helpers.ts";

// playwright.staging.config.ts points these at the deployed staging pair
const OLD = process.env.HF_MOVE_OLD ?? "http://[::1]:5173";
const NEW = process.env.HF_MOVE_NEW ?? "http://localhost:5173";
const MODE = NEW.startsWith("https:") ? "staging" : "development";
// the current key and version, so a version bump needs no change here
const STORAGE = fs.readFileSync("frontend/Stores/ConfigStorage.ts", "utf8");
const SAVE_KEY = STORAGE.match(/STORAGE_KEY = "(.+)"/)[1];
const VERSION = Number(STORAGE.match(/STORAGE_VERSION = (\d+)/)[1]);
const BACKUP = fs.readFileSync(
  new URL("../honing-forecast-2026-10-17.json", import.meta.url),
  "utf8",
);
const SAVE = LZString.compressToUTF16(BACKUP);
const FIRST_CHAR = JSON.parse(BACKUP).profiles[0].char_name;

let errors: string[];
// what the app reported to the Worker's /moves (Utils/MoveReport.ts), stubbed so nothing reaches D1
let reports: Record<string, unknown>[];
const outcomes = () => reports.map((r) => r.outcome);
test.beforeEach(async ({ page }) => {
  errors = hf.watch_errors(page);
  reports = [];
  await hf.stub_market(page);
  await page.route(hf.worker_route("/moves$"), (r) => {
    if (r.request().method() === "POST")
      reports.push(r.request().postDataJSON());
    return r.fulfill({
      body: "ok",
      headers: {
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Headers": "Content-Type",
      },
    });
  });
  // Cloudflare Web Analytics, injected on the deployed sites; its beacon logs a CORS error on pages.dev
  await page.route(/cloudflareinsights\.com/, (r) =>
    r.fulfill({ contentType: "text/javascript", body: "" }),
  );
  for (const site of [OLD, NEW])
    await page.route(`${site}/seed`, (r) =>
      r.fulfill({ contentType: "text/html", body: "<html></html>" }),
    );
});
test.afterEach(() => expect(errors).toEqual([]));

// a blank page on the site, so its storage can be read or written without the app running
async function storage(
  page: Page,
  site: string,
  set: Record<string, string> = {},
) {
  await page.goto(`${site}/seed`);
  return page.evaluate((set) => {
    for (const [k, v] of Object.entries(set)) localStorage.setItem(k, v);
    return { ...localStorage } as Record<string, string>;
  }, set);
}

async function arrive(page: Page, path: string) {
  await page.goto(OLD + path);
  await page.waitForURL(NEW + path);
  await expect(page.getByRole("link", { name: "Roster setup" })).toBeVisible();
}

const first_char = (page: Page) =>
  page.getByRole("link", { name: `Go to character ${FIRST_CHAR}` });
const moved_notice = (page: Page) =>
  page.getByRole("status").filter({ hasText: "has moved to" });

test("nothing saved: straight to the same path, as if typed in", async ({
  page,
}) => {
  const hops: string[] = [];
  page.on(
    "framenavigated",
    (f) => f === page.mainFrame() && hops.push(f.url()),
  );
  await arrive(page, "/roster-setup");
  await arrive(page, "/roster-setup"); // through the old link again
  expect(hops.filter((u) => u.includes("/migrate/"))).toEqual([]);
  expect((await storage(page, OLD)).hf_moved).toBe("1");
  await expect(moved_notice(page)).toHaveCount(0);
  expect(outcomes()).toEqual([]);
});

test("a save moves over, then later visits redirect straight away", async ({
  page,
}) => {
  await storage(page, OLD, { [SAVE_KEY]: SAVE, scanner_settings: '{"a":1}' });
  await arrive(page, "/roster-setup");
  await expect(first_char(page)).toBeVisible();
  await expect(moved_notice(page)).toBeVisible();
  const moved = await storage(page, NEW);
  expect(moved.scanner_settings).toBe('{"a":1}');
  expect(moved.hf_moved_in).toBeUndefined();
  expect((await storage(page, OLD)).hf_moved).toBe("1");
  await expect.poll(outcomes).toEqual(["moved"]);
  expect(reports[0]).toMatchObject({ chunks: 1, version: MODE });

  const hops: string[] = [];
  page.on(
    "framenavigated",
    (f) => f === page.mainFrame() && hops.push(f.url()),
  );
  await arrive(page, "/roster-setup");
  expect(hops.filter((u) => u.includes("/migrate/")).length).toBe(0);
});

test("the new site's untouched default save is replaced", async ({ page }) => {
  await hf.open(page, NEW + "/roster-setup");
  await page.goto(`${NEW}/seed`); // pagehide writes the default save
  expect((await storage(page, NEW))[SAVE_KEY]).toBeDefined();
  await storage(page, OLD, { [SAVE_KEY]: SAVE });
  await arrive(page, "/roster-setup");
  await expect(first_char(page)).toBeVisible();
  expect(outcomes()).toEqual(["moved"]);
});

test("the new site's own save stays; the old one can be used instead", async ({
  page,
}) => {
  const own = JSON.parse(BACKUP);
  own.profiles = [{ ...own.profiles[1] }];
  own.snapshots = [];
  await storage(page, NEW, {
    [SAVE_KEY]: LZString.compressToUTF16(JSON.stringify(own)),
  });
  await storage(page, OLD, { [SAVE_KEY]: SAVE });
  await arrive(page, "/roster-setup");
  const banner = page.getByRole("alert").filter({ hasText: "was not loaded" });
  await expect(banner).toBeVisible();
  await expect(first_char(page)).toHaveCount(0);

  const download = page.waitForEvent("download");
  await banner.getByRole("button", { name: "Download" }).click();
  const file = await (await download).path();
  expect(JSON.parse(fs.readFileSync(file, "utf8")).profiles[0].char_name).toBe(
    FIRST_CHAR,
  );

  await banner.getByRole("button", { name: "Use it instead" }).click();
  // a full reload, slow in Firefox under parallel load
  await expect(first_char(page)).toBeVisible({ timeout: 20_000 });
  await expect(banner).toHaveCount(0);
  await expect
    .poll(outcomes)
    .toEqual(["conflict", "chose_download", "chose_old"]);
});

// a visitor who last used the old site long ago: their save, under its own version's key,
// arrives on the new site and is migrated to the current version there
for (const [fixture, version] of [
  ["V7 config", 7],
  ["v8 config", 8],
  ["V10 config", 10],
  ["V11 config", 11],
  ["v12 config.json", 12],
  ["v13 config.json", 13],
] as const) {
  test(`a real V${version} save moves and is migrated`, async ({ page }) => {
    const raw = fs.readFileSync(`tests/${fixture}`, "utf8");
    const json = raw.startsWith("{") ? raw : LZString.decompressFromUTF16(raw);
    const profiles = JSON.parse(json).profiles.length;
    const old_key = `HF_CONFIG_V${version}_COMPRESSED`;
    await storage(page, OLD, { [old_key]: LZString.compressToUTF16(json) });
    await arrive(page, "/roster-setup");
    await expect(
      page.getByRole("link", { name: "Go to character Toneema" }),
    ).toBeVisible();
    await expect.poll(outcomes).toEqual(["moved"]);
    const moved = await storage(page, NEW); // leaving the app saves it
    expect(moved[old_key]).toBeUndefined();
    const saved = JSON.parse(LZString.decompressFromUTF16(moved[SAVE_KEY]));
    expect(saved.version).toBe(VERSION);
    expect(saved.profiles).toHaveLength(profiles);
  });
}

test("a large save moves in several chunks", async ({ page }) => {
  // incompressible, so ~3 chunks of 500k
  const big = crypto.randomBytes(900_000).toString("base64");
  await storage(page, OLD, { [SAVE_KEY]: SAVE, scanner_font: big });
  const chunks: string[] = [];
  page.on(
    "framenavigated",
    (f) =>
      f === page.mainFrame() && f.url().includes("#n=") && chunks.push(f.url()),
  );
  await arrive(page, "/roster-setup");
  expect(chunks.length).toBeGreaterThan(1);
  expect((await storage(page, NEW)).scanner_font).toBe(big);
});

test("a hand-made link imports nothing", async ({ page }) => {
  await page.goto(`${NEW}/migrate/#n=0&of=1&len=4&to=%2Froster-setup&d=AAAA`);
  await page.waitForURL(NEW + "/roster-setup");
  const stored = await storage(page, NEW);
  expect(stored.hf_imported).toBeUndefined();
  expect(stored.hf_moved_in).toBeUndefined();
});

test("a failed move says so; ?stay keeps the old site open", async ({
  page,
}) => {
  // a chunk that isn't valid data, sent from the old site
  await page.goto(`${OLD}/seed`);
  await page.evaluate(
    (to) => location.replace(to),
    `${NEW}/migrate/#n=0&of=1&len=4&to=%2Froster-setup&d=AAAA`,
  );
  await page.waitForURL(NEW + "/roster-setup");
  const banner = page
    .getByRole("alert")
    .filter({ hasText: "couldn't be moved" });
  await expect(banner).toBeVisible();
  errors.length = 0; // the receiver logs the failure
  await expect.poll(outcomes).toEqual(["failed"]);
  expect(reports[0].error).toBeTruthy();

  await banner.getByRole("link", { name: "Use this link" }).click();
  await page.waitForURL(`${OLD}/roster-setup?stay`);
  await expect(page.getByRole("link", { name: "Roster setup" })).toBeVisible();
  expect(page.url()).toBe(`${OLD}/roster-setup?stay`);
});

test("gives up sending after 3 visits", async ({ page }) => {
  await storage(page, OLD, { [SAVE_KEY]: SAVE });
  await page.route(`${NEW}/migrate/`, (r) =>
    r.fulfill({ contentType: "text/html", body: "<html></html>" }),
  );
  for (let i = 0; i < 3; i++) {
    await page.goto(OLD + "/roster-setup");
    await page.waitForURL(/#n=0/);
  }
  await page.unroute(`${NEW}/migrate/`);
  await page.goto(OLD + "/roster-setup");
  await page.waitForURL(NEW + "/roster-setup");
  await expect.poll(outcomes).toEqual(["gave_up"]);
});
