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
  await hf.open(page, `/${name}/scanner`);
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
    useRosterStore().add_profile({ ...new_char_profile(), char_name: "Bee" });
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

  // Bee: roster storage is covered and taken from Newchar's export
  await open_scanner(page, "Bee");
  const cover = page.getByRole("group", { name: "Roster storage covered" });
  await expect(cover).toBeVisible();
  await expect(cover).toContainText("Newchar");
  await expect(page.getByRole("button", { name: "1 saved" })).toBeVisible();
  await scan(
    page,
    [
      ["Roster", 0, 0, "Red", 999, "Tradable"],
      ["CharInventory", 0, 0, "Blue", 7, "RosterBound"],
      ["CharInventory", 0, 1, "Blue", 3, "CharBound"],
    ],
    ["Roster 0", "CharInventory 0"],
  );
  // the covered read is ignored; Newchar's roster bound is added
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("100");
  await expect(manifest_cell(page, "Blue", "Roster bound")).toHaveText("27");
  await expect(page.getByRole("group", { name: "Saved pages" })).toContainText(
    "Newchar",
  );
  await page.getByRole("button", { name: "Export to calculator" }).click();
  now = await budgets(page);
  expect([now.bound.Blue, now.roster.Blue, now.tradable.Red]).toEqual([
    3, 27, 100,
  ]);

  // uncovering it lets the read count, and the button covers it again
  await scan(page, [], ["Roster 0"]);
  await page.getByRole("button", { name: "Scan roster storage" }).click();
  await expect(cover).toHaveCount(0);
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("999");
  await page.getByRole("button", { name: "Use saved" }).click();
  await expect(cover).toBeVisible();
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("100");
  await page.waitForTimeout(1000);

  // snapshots: from Newchar's first, before export, to now
  await hf.open(page, "/snapshots");
  const list = page.getByRole("group", { name: "Snapshot list" });
  // Bee's "before" is Newchar's "after", so it is not added
  await expect(list.getByRole("button", { name: /^Delete / })).toHaveCount(3);
  await list
    .getByRole("radio", { name: /^From .* Before export$/ })
    .first()
    .click();
  // "First" is the newest: pick the oldest
  await list
    .getByRole("radio", { name: /^From .* Before export$/ })
    .last()
    .click();
  const changes = page.getByRole("group", { name: "Snapshot changes" });
  await expect(changes.getByLabel("Blue Roster bound")).toHaveText("+27");
  await expect(changes.getByLabel("Blue Char bound")).toHaveText("+53");
  await page.getByLabel("Character").selectOption({ label: "Bee" });
  await expect(changes.getByLabel("Blue Char bound")).toHaveText("+3");
  // a manual snapshot equal to the latest is not added
  await page.getByRole("button", { name: "Take snapshot" }).click();
  await expect(list.getByRole("button", { name: /^Delete / })).toHaveCount(3);
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
    page.getByRole("banner").getByRole("link", { name, exact: true });
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

  // exported, so leaving wipes it without asking
  await link("Newchar").click();
  await expect(page).toHaveURL(/\/Newchar\/calc$/);
  await expect(dialog).toHaveCount(0);
  expect(await scan_edits(page)).toBe(0);

  // wiped, the switch goes where it was headed and Bee keeps its export
  await page.getByRole("link", { name: "Scanner", exact: true }).click();
  await expect(page).toHaveURL(/\/Newchar\/scanner$/);
  await scan(page, blue(5), ["CharInventory 0"]);
  await link("Bee").click();
  await dialog.getByRole("button", { name: "Wipe the scan" }).click();
  await expect(page).toHaveURL(/\/Bee\/calc$/);
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

test("a scan moved to another roster takes that roster's cover", async ({
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
      roster_id: 1,
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

  // Newchar's uncovered, moved to Cee: Cee's roster is covered by its own default again
  await page.getByRole("button", { name: "Scan roster storage" }).click();
  await scan(page, red(999), ["Roster 0"]);
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("999");
  await page
    .getByRole("banner")
    .getByRole("link", { name: "Cee", exact: true })
    .click();
  const dialog = page.getByRole("dialog", { name: "Switch character" });
  await expect(dialog).toContainText("Cee is on another roster");
  await dialog.getByRole("button", { name: "Move the scan to Cee" }).click();
  await expect(page).toHaveURL(/\/Cee\/scanner$/);
  await expect(
    page.getByRole("group", { name: "Roster storage covered" }),
  ).toBeVisible();
  await expect(manifest_cell(page, "Red", "Tradable")).toHaveText("7");
});

test("chests are edited on the calculator page", async ({ page }) => {
  await hf.open(page, "/Newchar/calc");
  await page.getByLabel("Edit chests").check();
  await page.getByLabel("New chest count").fill("3");
  await page.getByLabel("New chest name").fill("Test box");
  await page.getByLabel("Option 1 material").selectOption("Red");
  await page.getByLabel("Option 1 amount").fill("10");
  await page.getByRole("button", { name: "Add chest" }).click();
  const count = page.getByLabel("Test box count");
  await expect(count).toHaveValue("3");
  await count.fill("5");
  await count.blur();
  await page.getByLabel("Edit chests").uncheck();
  await expect(page.getByRole("group", { name: /Obtain-all/ })).toContainText(
    "Test box",
  );
  await expect(
    page
      .getByRole("group", { name: /Obtain-all/ })
      .getByLabel("Test box count"),
  ).toHaveValue("5");
  await page.getByLabel("Edit chests").check();
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

// what the scanner made of each page, as the manifest has it
async function live_pages(page: Page): Promise<Pages> {
  return page.evaluate(async () => {
    const url = performance
      .getEntriesByType("resource")
      .map((entry) => entry.name)
      .filter((name) => name.includes("InventoryScanner/Manifest.ts"))
      .pop();
    const { live } = await import(url);
    return JSON.parse(JSON.stringify(live.value.pages));
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

const OWN = (key: string) =>
  !["Roster", "Currency"].includes(key.split(" ")[0]);

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
  await page.getByRole("button", { name: "Export to calculator" }).click();
  // saving is debounced
  await page.waitForTimeout(1000);
  return budgets(page);
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
  const cover = page.getByRole("group", { name: "Roster storage covered" });

  // character 1: roster storage, storage page 1, inventory page 1
  const one = await replay(page, "Newchar", CAPTURES[0]);
  await expect(cover).toHaveCount(0);
  let now = await export_now(page);
  expect(sorted(now.bound)).toEqual(sorted(total([one], 0, OWN)));
  expect(sorted(now.roster)).toEqual(sorted(total([one], 1)));
  expect(sorted(now.tradable)).toEqual(sorted(total([one], 2)));

  // character 2: roster storage covered by character 1's, its own pages added to the roster
  const two = await replay(page, "Bee", CAPTURES[1]);
  await expect(cover).toBeVisible();
  await expect(cover).toContainText("Newchar");
  now = await export_now(page);
  const after_two = now;
  expect(sorted(now.bound)).toEqual(sorted(total([two], 0, OWN)));
  expect(sorted(now.roster)).toEqual(
    sorted(add(total([one], 1), total([two], 1, OWN))),
  );
  expect(sorted(now.tradable)).toEqual(
    sorted(add(total([one], 2), total([two], 2, OWN))),
  );

  // only the inventory, which has not changed: storage comes from the last export, nothing moves
  await replay(page, "Bee", CAPTURES[2]);
  const storage = page.getByRole("group", { name: "Character storage" });
  await expect(storage.getByRole("button", { name: "1 saved" })).toBeVisible();
  await expect(storage.getByRole("button", { name: "2 saved" })).toBeVisible();
  await expect(
    page
      .getByRole("group", { name: "Inventory" })
      .getByRole("img", { name: "saved" }),
  ).toHaveCount(0);
  await expect(page.getByRole("group", { name: "Saved pages" })).toContainText(
    "Character storage page 1",
  );
  now = await export_now(page);
  expect(now).toEqual(after_two);

  // everything, roster storage page 2 too: covered, it is ignored; uncovered, it counts
  let four = await replay(page, "Bee", CAPTURES[3]);
  await expect(cover).toBeVisible();
  await expect(
    page
      .getByRole("group", { name: "Roster storage" })
      .getByRole("button", { name: "1 saved" }),
  ).toBeVisible();
  await expect(storage.getByRole("img", { name: "saved" })).toHaveCount(0);
  // covered, what is read there is left out
  expect(Object.keys(four).some((key) => key.startsWith("Roster"))).toBe(false);
  await page.getByRole("button", { name: "Scan roster storage" }).click();
  await expect(cover).toHaveCount(0);
  four = await live_pages(page);
  now = await export_now(page);
  const page_2 = (key: string) => key === "Roster 1";
  expect(sorted(now.bound)).toEqual(sorted(after_two.bound));
  expect(sorted(now.roster)).toEqual(
    sorted(add(after_two.roster, total([four], 1, page_2))),
  );
  expect(sorted(now.tradable)).toEqual(
    sorted(add(after_two.tradable, total([four], 2, page_2))),
  );
  // roster storage page 2 holds tradables only
  expect(Object.keys(total([four], 2, page_2)).length).toBeGreaterThan(0);

  // the snapshots: the empty start, and one after each export that changed something
  await hf.open(page, "/snapshots");
  const list = page.getByRole("group", { name: "Snapshot list" });
  await expect(list.getByRole("button", { name: /^Delete / })).toHaveCount(4);
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
    .getByRole("banner")
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
