import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import * as hf from "./helpers.ts";

// Exports from two characters of one roster, merged through saved pages, then the snapshots and
// chest editing. Scans are injected into the scanner page's store rather than captured.

let errors: string[];
test.beforeEach(async ({ page }) => {
  errors = hf.watch_errors(page);
  await hf.stub_market(page);
});
test.afterEach(() => expect(errors).toEqual([]));

type Put = [string, number, number, string, number, string];

// window, page, column, item, amount, tradability; and the pages open in game
async function scan(page: Page, puts: Put[], seen: string[]) {
  await page.evaluate(
    async ({ puts, seen }) => {
      const url = performance
        .getEntriesByType("resource")
        .map((entry) => entry.name)
        .filter((name) => name.includes("InventoryScanner/ScanStore.ts"))
        .pop();
      const store = await import(url);
      for (const key of seen) store.seen_pages.value[key] = true;
      for (const [window, num, col, item, amount, tradability] of puts) {
        store.set_edit({
          address: {
            inventory_type: window,
            page_num: num,
            pos_in_inv: [0, col],
          },
          item,
          amount,
          tradability,
        });
      }
    },
    { puts, seen },
  );
}

// the roster's two columns and the active character's bound one, as numbers
async function budgets(page: Page) {
  return page.evaluate(async () => {
    const url = performance
      .getEntriesByType("resource")
      .map((entry) => entry.name)
      .filter((name) => name.includes("Stores/RosterConfig.ts"))
      .pop();
    const { useRosterStore } = await import(url);
    const store = useRosterStore();
    const num = (column) =>
      Object.fromEntries(
        Object.entries(column.values).map(([k, v]: [string, string]) => [
          k,
          Number(v.replace(/\D/g, "")),
        ]),
      );
    return {
      bound: num(store.active_profile.bound_budgets),
      roster: num(store.active_roster_mats_owned),
      tradable: num(store.active_tradable_mats_owned),
      pages: store.roster_config.scan_pages.length,
      snapshots: store.roster_config.snapshots.length,
    };
  });
}

async function open_scanner(page: Page, name: string) {
  await hf.open(page, `/chars/${name}/scanner`);
  await expect(page.getByRole("button", { name: "Share screen" })).toBeVisible({
    timeout: 60_000,
  });
}

async function add_bee(page: Page) {
  await page.evaluate(async () => {
    const find = (part) =>
      performance
        .getEntriesByType("resource")
        .map((entry) => entry.name)
        .filter((name) => name.includes(part))
        .pop();
    const { useRosterStore } = await import(find("Stores/RosterConfig.ts"));
    const { new_char_profile } = await import(
      find("Stores/CharacterProfile.ts")
    );
    const store = useRosterStore();
    store.add_profile({
      ...new_char_profile(),
      char_name: "Bee",
      roster_id: store.active_profile.roster_id,
    });
  });
  // saving is debounced
  await page.waitForTimeout(1000);
}

const manifest_cell = (page: Page, label: string, band: string) =>
  page
    .getByRole("group", { name: "Manifest materials" })
    .getByLabel(`${label} ${band}`, { exact: true });

test("exports merge across characters and are snapshotted", async ({
  page,
}) => {
  test.setTimeout(180_000);
  await open_scanner(page, "Newchar");
  await add_bee(page);

  // Newchar: roster storage, own storage and inventory
  await expect(
    page.getByRole("group", { name: "Roster storage covered" }),
  ).toHaveCount(0);
  await scan(
    page,
    [
      ["Roster", 0, 0, "Red", 100, "Tradable"],
      ["CharStorage", 0, 0, "Blue", 50, "CharBound"],
      ["CharInventory", 0, 0, "Blue", 20, "RosterBound"],
    ],
    ["Roster 0", "CharStorage 0", "CharInventory 0"],
  );
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("100");
  await page.getByRole("button", { name: "Export to calculator" }).click();
  let now = await budgets(page);
  expect([now.bound.Blue, now.roster.Blue, now.tradable.Red]).toEqual([
    50, 20, 100,
  ]);
  expect(now.snapshots).toBe(2);
  await page.waitForTimeout(1000);

  // Bee: roster storage, not opened yet, is taken from Newchar's export
  await open_scanner(page, "Bee");
  const cover = page.getByRole("group", { name: "Roster storage covered" });
  await expect(cover).toBeVisible();
  await expect(cover).toContainText("Newchar");
  await expect(
    page.getByText("Not opened in game, taken as unchanged:").locator(".."),
  ).toContainText("Roster storage page 1");
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("100");
  await scan(
    page,
    [
      ["Roster", 0, 0, "Red", 999, "Tradable"],
      ["CharInventory", 0, 0, "Blue", 7, "RosterBound"],
      ["CharInventory", 0, 1, "Blue", 3, "CharBound"],
    ],
    ["Roster 0", "CharInventory 0"],
  );
  // opened, what Bee reads counts; Newchar's roster bound is added
  await expect(cover).toHaveCount(0);
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("999");
  await expect(manifest_cell(page, "Blue", "Roster bound")).toHaveText("27");
  await page.getByRole("button", { name: "Export to calculator" }).click();
  now = await budgets(page);
  expect([now.bound.Blue, now.roster.Blue, now.tradable.Red]).toEqual([
    3, 27, 999,
  ]);
  await page.waitForTimeout(1000);

  // snapshots on Bee's income page: Bee's export against Newchar's
  await hf.open(page, "/chars/Bee/income");
  const timeline = page.getByRole("group", { name: "Snapshots" });
  // Bee's "before" is Newchar's "after", so it is not added
  await expect(timeline.getByRole("button")).toHaveCount(3);
  await timeline
    .getByRole("button", { name: /After export$/ })
    .last()
    .click();
  const dashboard = page.getByRole("group", { name: "Snapshot dashboard" });
  await expect(dashboard.getByLabel("Blue Roster bound")).toHaveText("+7");
  await expect(dashboard.getByLabel("Blue Char bound")).toHaveText("+3");
  // nothing changed since the latest, so a manual snapshot is not offered
  await expect(
    page.getByRole("button", { name: "Take snapshot" }),
  ).toBeDisabled();
});

test("a page not opened is taken from the last export", async ({ page }) => {
  test.setTimeout(180_000);
  await open_scanner(page, "Newchar");
  await add_bee(page);
  await scan(page, [["Roster", 0, 0, "Red", 100, "Tradable"]], ["Roster 0"]);
  await page.getByRole("button", { name: "Export to calculator" }).click();
  await page.waitForTimeout(1000);

  // Bee opens page 2 only: page 1 stays covered and counts as Newchar left it
  await open_scanner(page, "Bee");
  const cover = page.getByRole("group", { name: "Roster storage covered" });
  const roster = page.getByRole("group", { name: "Roster storage" });
  await expect(cover).toBeVisible();
  await scan(page, [["Roster", 1, 0, "Red", 30, "Tradable"]], ["Roster 1"]);
  await roster.getByRole("button", { name: "2", exact: true }).click();
  await expect(cover).toHaveCount(0);
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("130");
  await roster.getByRole("button", { name: "1", exact: true }).click();
  await expect(cover).toBeVisible();

  // a roster page never exported and not opened has no cover
  await page.getByRole("button", { name: "Export to calculator" }).click();
  expect((await budgets(page)).tradable.Red).toBe(130);
  await expect(cover).toBeVisible();
  await page.waitForTimeout(1000);
});

// how many slots the scan has corrections for, and how many it has read
async function scan_store(page: Page) {
  return page.evaluate(async () => {
    const url = performance
      .getEntriesByType("resource")
      .map((entry) => entry.name)
      .filter((name) => name.includes("InventoryScanner/ScanStore.ts"))
      .pop();
    const store = await import(url);
    return [Object.keys(store.edits.value).length, store.slots.value.size];
  });
}
const scan_edits = async (page: Page) => (await scan_store(page))[0];
const scan_size = async (page: Page) => (await scan_store(page))[1];

test("switching character asks what to do with an unexported scan", async ({
  page,
}) => {
  test.setTimeout(120_000);
  await open_scanner(page, "Newchar");
  await add_bee(page);
  const link = (name: string) =>
    page
      .getByRole("navigation", { name: "Characters" })
      .getByRole("link", { name, exact: true });
  const dialog = page.getByRole("dialog", { name: "Switch character" });
  const blue = (amount: number): Put[] => [
    ["CharInventory", 0, 0, "Blue", amount, "CharBound"],
  ];
  await scan(page, blue(20), ["CharInventory 0"]);

  // cancelled, nothing happens
  await link("Bee").click();
  await expect(dialog).toContainText("The scan on Newchar");
  await dialog.getByRole("button", { name: "Cancel" }).click();
  await expect(dialog).toHaveCount(0);
  await expect(page).toHaveURL(/\/Newchar\/scanner$/);
  expect(await scan_edits(page)).toBe(1);

  // moved, it carries on on Bee's scanner page and exports to Bee
  await link("Bee").click();
  await dialog.getByRole("button", { name: "Move the scan to Bee" }).click();
  await expect(page).toHaveURL(/\/Bee\/scanner$/);
  await expect(manifest_cell(page, "Blue", "Char bound")).toHaveText("20");
  await page.getByRole("button", { name: "Export to calculator" }).click();
  expect((await budgets(page)).bound.Blue).toBe(20);

  // exported, so leaving wipes it without asking; the tab keeps the page
  await link("Newchar").click();
  await expect(page).toHaveURL(/\/Newchar\/scanner$/);
  await expect(dialog).toHaveCount(0);
  expect(await scan_edits(page)).toBe(0);

  // wiped, the switch goes to that character's scanner and Bee keeps its export
  await scan(page, blue(5), ["CharInventory 0"]);
  await link("Bee").click();
  await dialog.getByRole("button", { name: "Wipe the scan" }).click();
  await expect(page).toHaveURL(/\/Bee\/scanner$/);
  expect(await scan_edits(page)).toBe(0);
  expect((await budgets(page)).bound.Blue).toBe(20);
});

test("reset scan wipes a stopped scan", async ({ page }) => {
  await open_scanner(page, "Newchar");
  const reset = page.getByRole("button", { name: "Reset scan" });
  await expect(reset).toBeDisabled();
  await scan(
    page,
    [["CharInventory", 0, 0, "Blue", 20, "CharBound"]],
    ["CharInventory 0"],
  );
  await expect(manifest_cell(page, "Blue", "Char bound")).toHaveText("20");
  await reset.click();
  expect(await scan_edits(page)).toBe(0);
  await expect(reset).toBeDisabled();
});

test("a custom number font needs every digit", async ({ page }) => {
  await open_scanner(page, "Newchar");
  await page.getByRole("button", { name: /Troubleshooting/ }).click();
  await page.getByLabel("Number font").selectOption("Custom");
  const apply = page.getByRole("button", { name: "Apply" });
  await expect(apply).toBeDisabled();
  const file = page.getByLabel("Font file");
  await file.setInputFiles({
    name: "not-a-font.ttf",
    mimeType: "font/ttf",
    buffer: Buffer.from("not a font"),
  });
  await expect(page.getByText("not-a-font.ttf is not a font")).toBeVisible();
  await expect(apply).toBeDisabled();
  await file.setInputFiles(
    fileURLToPath(new URL("../../public/fonts/English.ttf", import.meta.url)),
  );
  await expect(page.getByText("English.ttf", { exact: true })).toBeVisible();
  await expect(page.getByText("not-a-font.ttf is not a font")).toBeHidden();
  await expect(apply).toBeEnabled();

  // applied, only its digits and "+" are kept, and a reload keeps it in view
  await apply.click();
  const stored = await page.evaluate(() => localStorage.getItem("scanner_font"));
  expect(stored.length).toBeGreaterThan(0);
  expect(stored.length).toBeLessThan(8000);
  await page.reload();
  await expect(page.getByLabel("Number font")).toHaveValue("Custom");
  await expect(page.getByText("English.ttf", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Troubleshooting/ }),
  ).toBeDisabled();
});

test("a scan moved to another roster counts its roster storage there", async ({
  page,
}) => {
  test.setTimeout(120_000);
  await open_scanner(page, "Newchar");
  await page.evaluate(async () => {
    const find = (part) =>
      performance
        .getEntriesByType("resource")
        .map((entry) => entry.name)
        .filter((name) => name.includes(part))
        .pop();
    const { useRosterStore, create_default_owned_input_column } = await import(
      find("Stores/RosterConfig.ts")
    );
    const { new_char_profile } = await import(
      find("Stores/CharacterProfile.ts")
    );
    // as the roster page adds a roster
    const store = useRosterStore();
    store.add_profile({
      ...new_char_profile(),
      char_name: "Cee",
      roster_id: "1",
    });
    store.roster_config.roster_mats_owned[1] =
      create_default_owned_input_column();
    store.roster_config.tradable_mats_owned[1] =
      create_default_owned_input_column();
    store.roster_config.all_regions[1] = "nae";
  });
  await page.waitForTimeout(1000);
  const red = (amount: number): Put[] => [
    ["Roster", 0, 0, "Red", amount, "Tradable"],
  ];
  const export_scan = async () => {
    await page.getByRole("button", { name: "Export to calculator" }).click();
    await page.waitForTimeout(1000);
  };

  // both rosters have a saved roster storage
  await open_scanner(page, "Cee");
  await scan(page, red(7), ["Roster 0"]);
  await export_scan();
  await open_scanner(page, "Newchar");
  await scan(page, red(100), ["Roster 0"]);
  await export_scan();

  // roster storage opened on Newchar, moved to Cee: it is Cee's roster storage now
  await scan(page, red(999), ["Roster 0"]);
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("999");
  await page
    .getByRole("banner")
    .getByRole("link", { name: "Roster 2", exact: true })
    .click();
  const dialog = page.getByRole("dialog", { name: "Switch character" });
  await expect(dialog).toContainText("Cee is on another roster");
  await dialog.getByRole("button", { name: "Move the scan to Cee" }).click();
  await expect(page).toHaveURL(/\/Cee\/scanner$/);
  await expect(
    page.getByRole("group", { name: "Roster storage covered" }),
  ).toHaveCount(0);
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("999");
});

test("chests are edited on the calculator page", async ({ page }) => {
  await hf.open(page, "/chars/Newchar/calc");
  await page.getByLabel("Edit chests").check();
  await page.getByLabel("New chest count").fill("3");
  await page.getByLabel("New chest name").fill("Test box");
  await page.getByLabel("Option 1 material").selectOption("Red");
  await page.getByLabel("Option 1 amount").fill("10");
  await page.getByRole("button", { name: "Add chest" }).click();
  // a one-option chest is obtain-all, listed only while all chests are shown
  await page.getByLabel(/Show all chests/).check();
  const count = page.getByLabel("Test box count");
  await expect(count).toHaveValue("3");
  await count.fill("5");
  await count.blur();
  await page.getByLabel("Edit chests").uncheck();
  await expect(page.getByRole("group", { name: /Obtain-all/ })).toContainText(
    "Test box",
  );
  // counts are only shown while editing
  await expect(
    page
      .getByRole("group", { name: /Obtain-all/ })
      .getByLabel("Test box count"),
  ).toHaveCount(0);
  await page.getByLabel("Edit chests").check();
  await expect(page.getByLabel("Test box count")).toHaveValue("5");
  await page.getByRole("button", { name: "Delete Test box" }).click();
  await expect(page.getByLabel("Test box count")).toHaveCount(0);
});

// Four local debug captures: the first of one character, the other three of a second one, in
// the order taken. Each shows other pages.
const RECORDINGS = new URL("../../ignored/recordings/", import.meta.url);
const CAPTURES = [
  "2026-10-19T10-18-21-138Z",
  "2026-10-19T10-20-24-576Z",
  "2026-10-19T10-21-04-596Z",
  "2026-10-19T10-23-03-086Z",
].map((name) => new URL(`honing-forecast-${name}.hfcap`, RECORDINGS));

type Pages = Record<string, { materials: Record<string, number[]> }>;
type Amounts = Record<string, number>;

// what the scanner made of each page opened, as the manifest has it, and the currencies
async function live_pages(page: Page): Promise<Pages> {
  return page.evaluate(async () => {
    const url = performance
      .getEntriesByType("resource")
      .map((entry) => entry.name)
      .filter((name) => name.includes("InventoryScanner/Manifest.ts"))
      .pop();
    const { live, manifest } = await import(url);
    const pages = Object.fromEntries(
      manifest.value.read.map((one) => [
        `${one.window} ${one.page}`,
        one.content,
      ]),
    );
    return JSON.parse(
      JSON.stringify({ ...pages, Currencies: live.value.currencies }),
    );
  });
}

async function replay(page: Page, name: string, capture: URL) {
  await open_scanner(page, name);
  await page
    .getByLabel("Upload image or video")
    .setInputFiles(fileURLToPath(capture));
  await expect(page.getByText("Replay finished")).toBeVisible({
    timeout: 300_000,
  });
  return live_pages(page);
}

// one band of the pages added up, of the pages `keep` keeps
function total(pages: Pages[], band: number, keep = (key: string) => true) {
  const out: Amounts = {};
  for (const one of pages) {
    for (const [key, content] of Object.entries(one)) {
      if (!keep(key)) continue;
      for (const [label, bands] of Object.entries(content.materials)) {
        if (bands[band]) out[label] = (out[label] ?? 0) + bands[band];
      }
    }
  }
  return out;
}

const add = (...all: Amounts[]) => {
  const out: Amounts = {};
  for (const one of all) {
    for (const [label, amount] of Object.entries(one))
      out[label] = (out[label] ?? 0) + amount;
  }
  return out;
};
const nonzero = (amounts: Amounts) =>
  Object.fromEntries(Object.entries(amounts).filter(([, amount]) => amount));
const sorted = (amounts: Amounts) =>
  Object.fromEntries(Object.entries(nonzero(amounts)).sort());

async function export_now(page: Page) {
  // Slots left unread hide the manifest; exporting without them sets them to irrelevant. What a
  // re-run reads right is checked against hand edits, not here.
  const force = page.getByRole("button", {
    name: "Show the manifest without these",
  });
  if (await force.isVisible()) await force.click();
  await page.getByRole("button", { name: "Export to calculator" }).click();
  // saving is debounced
  await page.waitForTimeout(1000);
  return budgets(page);
}

const pick = (pages: Pages, keep: (key: string) => boolean) =>
  Object.fromEntries(Object.entries(pages).filter(([key]) => keep(key)));
const in_roster = (key: string) => key.startsWith("Roster ");

// What an export writes: the pages read now over this character's and the roster's saved ones,
// and the roster-wide bands of the other characters' saved pages.
function exported(now: Pages, saved: Pages, others: Pages) {
  const mine = { ...saved, ...now };
  return {
    bound: total([mine], 0),
    roster: add(total([mine], 1), total([others], 1)),
    tradable: add(total([mine], 2), total([others], 2)),
  };
}

test("captures of two characters merge as expected", async ({
  page,
  browserName,
}) => {
  test.skip(
    !CAPTURES.every((capture) => fs.existsSync(capture)) ||
      browserName !== "chromium",
    "local recordings, replayed in one browser",
  );
  test.setTimeout(900_000);
  await open_scanner(page, "Newchar");
  await add_bee(page);
  // the pages each export saved, by owner
  const saves: Record<string, Pages> = { roster: {}, Newchar: {}, Bee: {} };
  const step = async (name: string, capture: URL) => {
    const now = await replay(page, name, capture);
    const other = name === "Newchar" ? "Bee" : "Newchar";
    const want = exported(
      now,
      { ...saves.roster, ...saves[name] },
      pick(saves[other], (key) => !in_roster(key)),
    );
    const got = await export_now(page);
    expect(sorted(got.bound)).toEqual(sorted(want.bound));
    expect(sorted(got.roster)).toEqual(sorted(want.roster));
    expect(sorted(got.tradable)).toEqual(sorted(want.tradable));
    for (const [key, content] of Object.entries(now)) {
      if (key === "Currencies") continue;
      saves[in_roster(key) ? "roster" : name][key] = content;
    }
    return now;
  };

  // character 1, then three captures of character 2, each showing other pages
  await step("Newchar", CAPTURES[0]);
  await step("Bee", CAPTURES[1]);
  // a saved page of character storage not opened is covered
  const three = await step("Bee", CAPTURES[2]);
  if (saves.Bee["CharStorage 0"] && !three["CharStorage 0"])
    await expect(
      page.getByRole("group", { name: "Character storage covered" }),
    ).toBeVisible();
  await step("Bee", CAPTURES[3]);

  // the snapshots: the empty start, and one after the exports that changed something
  await hf.open(page, "/chars/Newchar/income");
  const timeline = page.getByRole("group", { name: "Snapshots" });
  expect(await timeline.getByRole("button").count()).toBeGreaterThanOrEqual(3);
});

// chests as what they give: their order, and that of a joined title, follow the reading
const unordered = (pages: Pages) =>
  Object.fromEntries(
    Object.entries(pages)
      .sort()
      .map(([key, content]: [string, any]) => [
        key,
        {
          ...content,
          chests: content.chests
            .map((chest) => `${chest.key} ${chest.band} ${chest.count}`)
            .sort(),
        },
      ]),
  );

test("reset scan mid-capture starts the capture over", async ({
  page,
  browserName,
}) => {
  test.skip(
    !fs.existsSync(CAPTURES[0]) || browserName !== "chromium",
    "local recordings, replayed in one browser",
  );
  test.setTimeout(900_000);
  const straight = await replay(page, "Newchar", CAPTURES[0]);

  await open_scanner(page, "Newchar");
  await page
    .getByLabel("Upload image or video")
    .setInputFiles(fileURLToPath(CAPTURES[0]));
  await expect
    .poll(() => scan_size(page), { timeout: 120_000 })
    .toBeGreaterThan(0);
  await page.getByRole("button", { name: "Reset scan" }).click();
  expect(await scan_size(page)).toBe(0);
  await expect(page.getByText("Replay finished")).toBeVisible({
    timeout: 300_000,
  });
  expect(unordered(await live_pages(page))).toEqual(unordered(straight));
});

test("a scan moved to another character mid-capture reads the same", async ({
  page,
  browserName,
}) => {
  test.skip(
    !fs.existsSync(CAPTURES[0]) || browserName !== "chromium",
    "local recordings, replayed in one browser",
  );
  test.setTimeout(900_000);
  const straight = await replay(page, "Newchar", CAPTURES[0]);

  await open_scanner(page, "Newchar");
  await add_bee(page);
  await page
    .getByLabel("Upload image or video")
    .setInputFiles(fileURLToPath(CAPTURES[0]));
  await expect
    .poll(() => scan_size(page), { timeout: 120_000 })
    .toBeGreaterThan(0);
  await page
    .getByRole("navigation", { name: "Characters" })
    .getByRole("link", { name: "Bee", exact: true })
    .click();
  await page
    .getByRole("dialog", { name: "Switch character" })
    .getByRole("button", { name: "Move the scan to Bee" })
    .click();
  await expect(page).toHaveURL(/\/Bee\/scanner$/);
  await expect(page.getByText("Replay finished")).toBeVisible({
    timeout: 300_000,
  });
  expect(unordered(await live_pages(page))).toEqual(unordered(straight));
});

// Captures whose slots were all set by hand in the end record (capture_edit), and the pages each
// must read.
const CHECKED = [
  {
    // a native 3440x1440 game, roster storage hovered
    name: "a native ultrawide capture",
    file: "2026-10-19T16-29-47-201Z",
    pages: ["CharInventory 0", "CharStorage 0", "Roster 0"],
  },
  {
    // 1080p, roster storage page 2, every slot checked by hand
    name: "a 1080p capture",
    file: "2026-10-20T11-43-25-861Z",
    pages: ["CharInventory 0", "CharStorage 0", "Roster 1"],
  },
  {
    // 1080p, November 2026 client: the Vestiges of the End clear chests
    name: "a capture with the November 2026 chests",
    file: "2026-11-18T15-58-07-564Z",
    pages: ["CharInventory 0", "CharStorage 0", "CharStorage 1", "Roster 0", "Roster 1"],
    // its one hover read "20,000 Gold Bars" as "220.000"; another hover would fix it
    misread: ["CharInventory 0 3,0"],
  },
];

// The slots set by hand in the capture's end record, the only right answers it has, that the
// replay read otherwise, as the native capture_replay --check holds them. A slot set with no item
// has to read as irrelevant.
async function wrong_edits(page: Page): Promise<string[]> {
  return page.evaluate(async () => {
    const url = performance
      .getEntriesByType("resource")
      .map((entry) => entry.name)
      .filter((name) => name.includes("InventoryScanner/ScanStore.ts"))
      .pop();
    const { slots } = await import(url);
    const edits = (globalThis as any).__capture_reference.edits;
    return Object.entries(edits).flatMap(([key, edit]: [string, any]) => {
      const slot = slots.value.get(key);
      const amount = edit.amount ?? 1;
      const right =
        edit.item == null
          ? slot?.status === "Irrelevant"
          : slot &&
            [
              slot.icon_name_score?.[0],
              ...slot.alternatives,
              slot.label,
              slot.item,
            ].includes(edit.item) &&
            slot.value === amount &&
            (edit.tradability == null ||
              slot.tradability === edit.tradability) &&
            ["Good", "NeedTradability"].includes(slot.status);
      return right
        ? []
        : [
            `${key}: set ${edit.item} x${amount} ${edit.tradability}, read ${slot?.icon_name_score?.[0]} x${slot?.value} ${slot?.tradability} ${slot?.status}`,
          ];
    });
  });
}

for (const checked of CHECKED) {
  const capture = new URL(`honing-forecast-${checked.file}.hfcap`, RECORDINGS);
  test(`${checked.name} reads its hand-set slots right`, async ({
    page,
    browserName,
  }) => {
    test.skip(
      !fs.existsSync(capture) || browserName !== "chromium",
      "local recordings, replayed in one browser",
    );
    test.setTimeout(900_000);
    await replay(page, "Newchar", capture);
    const read = await page.evaluate(async () => {
      const url = performance
        .getEntriesByType("resource")
        .map((entry) => entry.name)
        .filter((name) => name.includes("InventoryScanner/Manifest.ts"))
        .pop();
      const { manifest } = await import(url);
      return manifest.value.read.map((one) => `${one.window} ${one.page}`);
    });
    expect(read.sort()).toEqual(checked.pages);
    const misread = checked.misread ?? [];
    const wrong = await wrong_edits(page);
    expect(wrong.filter((x) => !misread.includes(x.split(":")[0]))).toEqual([]);
  });
}
