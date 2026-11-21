// Roster setup and the market page's per-roster mats; the effects are read from the store and after a reload.
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import fs from "node:fs";
import * as hf from "./helpers.ts";
import * as bh from "./bible_helpers.ts";

let errors: string[];
test.beforeEach(async ({ page }) => {
  errors = hf.watch_errors(page);
  await hf.stub_market(page);
  // signed out: nothing may reach lostark.bible
  await bh.stub_bible(page, []);
});
test.afterEach(() => expect(errors).toEqual([]));

const market = JSON.parse(
  fs.readFileSync(new URL("./fixtures/market.json", import.meta.url), "utf8"),
);
const persist = (page: Page) => page.waitForTimeout(1000);
const roster = (page: Page, n: number) =>
  page.getByRole("group", { name: `Roster ${n}` });
const header_tab = (page: Page, name: string) =>
  page.locator("header").getByRole("link", { name, exact: true });

async function open_roster(page: Page, rosters: bh.SeedRoster[]) {
  await hf.open(page, "/roster-setup");
  await bh.seed(page, rosters);
}
// typing a name starts a fetch popup (scraped, stubbed as not found); close it
async function rename(page: Page, from: string, to: string) {
  const box = page.locator(`input[aria-label="Char name"]`);
  const i = (await bh.roster_names(page)).flat().indexOf(from);
  await box.nth(i).fill(to);
  await box.nth(i).blur();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await page.mouse.click(3, 3);
  await expect(dialog).toHaveCount(0);
}

test("Add new roster adds a header tab that survives a reload", async ({
  page,
}) => {
  await hf.open(page, "/roster-setup");
  await expect(header_tab(page, "Character")).toBeVisible();
  await page.getByRole("button", { name: "Add new roster" }).first().click();
  await expect(header_tab(page, "Roster 2")).toBeVisible();
  await expect(roster(page, 2)).toBeVisible();
  await persist(page);
  await page.reload();
  await expect(header_tab(page, "Roster 1")).toBeVisible();
  await expect(header_tab(page, "Roster 2")).toBeVisible();
  expect(await bh.roster_names(page)).toEqual([["Newchar"], ["Newchar1"]]);
});

test("Add new character adds a row to that roster only", async ({ page }) => {
  await open_roster(page, [{ names: ["Alpha"] }, { names: ["Beta"] }]);
  await roster(page, 1)
    .getByRole("button", { name: "Add new character" })
    .click();
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Alpha", "Newchar"], ["Beta"]]);
  await persist(page);
  await page.reload();
  expect(await bh.roster_names(page)).toEqual([["Alpha", "Newchar"], ["Beta"]]);
});

test("renaming formats the name; the link, the URL and a reload follow", async ({
  page,
}) => {
  await open_roster(page, [{ names: ["Alpha"] }]);
  await rename(page, "Alpha", "bob smith");
  await expect(page.locator(`input[aria-label="Char name"]`)).toHaveValue(
    "Bobsmith",
  );
  await expect.poll(() => bh.roster_names(page)).toEqual([["Bobsmith"]]);
  await page.getByRole("link", { name: "Go to character Bobsmith" }).click();
  await expect(page).toHaveURL(/\/chars\/NA\/Bobsmith\/calc$/);
  await persist(page);
  await page.reload();
  await expect(page).toHaveURL(/\/chars\/NA\/Bobsmith\/calc$/);
  await expect(page.getByRole("textbox", { name: "Char name" })).toHaveValue(
    "Bobsmith",
  );
});

test("renaming onto a name used in another roster gets a suffix", async ({
  page,
}) => {
  await open_roster(page, [{ names: ["Alpha"] }, { names: ["Beta"] }]);
  await rename(page, "Beta", "alpha");
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Alpha"], ["Alpha1"]]);
  await roster(page, 2)
    .getByRole("link", { name: "Go to character Alpha1" })
    .click();
  await expect(page).toHaveURL(/\/chars\/NA\/Alpha1\/calc$/);
  expect(
    await bh.with_store(page, (s) =>
      s.roster_ids.indexOf(s.active_profile.roster_id),
    ),
  ).toBe(1);
});

test("Move up and down reorder one roster and persist", async ({ page }) => {
  await open_roster(page, [{ names: ["A", "B", "C"] }, { names: ["D", "E"] }]);
  await page.getByRole("button", { name: "Move B up" }).click();
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([
      ["B", "A", "C"],
      ["D", "E"],
    ]);
  await persist(page);
  await page.reload();
  expect(await bh.roster_names(page)).toEqual([
    ["B", "A", "C"],
    ["D", "E"],
  ]);
  await page.getByRole("button", { name: "Move D down" }).click();
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([
      ["B", "A", "C"],
      ["E", "D"],
    ]);
  await persist(page);
  await page.reload();
  expect(await bh.roster_names(page)).toEqual([
    ["B", "A", "C"],
    ["E", "D"],
  ]);
});

test("Make a copy carries the ticked cells", async ({ page }) => {
  await hf.open(page, "/chars/NA/Newchar/calc");
  await hf.toggle_column(page, "normal", 15);
  await page.goto("/roster-setup");
  await page.getByRole("button", { name: "Make a copy of Newchar" }).click();
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Newchar", "Newchar1"]]);
  await page.getByRole("link", { name: "Go to character Newchar1" }).click();
  await expect(page).toHaveURL(/\/chars\/NA\/Newchar1\/calc$/);
  expect(await hf.cell_status(page, "normal", "Helmet", 15)).toBe("Want");
  expect(await hf.cell_status(page, "normal", "Helmet", 16)).toBe("NotYet");
  await persist(page);
  await page.reload();
  expect(await hf.cell_status(page, "normal", "Helmet", 15)).toBe("Want");
});

test("deleting a roster's last character removes the roster everywhere", async ({
  page,
}) => {
  await open_roster(page, [{ names: ["A"] }, { names: ["B"] }]);
  await page.goto("/market-mats");
  await expect(
    page.getByRole("combobox", { name: "Active roster" }),
  ).toBeVisible();
  await page.goto("/roster-setup");
  await roster(page, 2).getByRole("button", { name: "Delete B" }).click();
  await expect.poll(() => bh.roster_names(page)).toEqual([["A"]]);
  await expect(header_tab(page, "Character")).toBeVisible();
  await expect(header_tab(page, "Roster 2")).toHaveCount(0);
  await persist(page);
  await page.reload();
  expect(await bh.roster_names(page)).toEqual([["A"]]);
  await page.getByRole("link", { name: "Market & Mats" }).click();
  await expect(
    page.getByRole("combobox", { name: "Active roster" }),
  ).toHaveCount(0);
  expect(
    await bh.with_store(
      page,
      (s) => Object.keys(s.roster_config.roster_mats_owned).length,
    ),
  ).toBe(1);
});

test("deleting the character you were viewing lands on a valid one", async ({
  page,
}) => {
  await open_roster(page, [{ names: ["A", "B"] }]);
  await page.goto("/chars/NA/B/calc");
  await page.getByRole("link", { name: "Roster setup" }).click();
  await page.getByRole("button", { name: "Delete B" }).click();
  await expect.poll(() => bh.roster_names(page)).toEqual([["A"]]);
  await header_tab(page, "Character").click();
  await expect(page).toHaveURL(/\/chars\/NA\/A\/calc$/);
  await expect(
    page.getByRole("group", { name: "Normal honing" }),
  ).toBeVisible();
  await page.goto("/chars/NA/B/calc");
  await expect(page).not.toHaveURL(/\/chars\/NA\/B\//);
});

test("each roster card has its own region", async ({ page }) => {
  const asked: string[] = [];
  page.on("request", (r) => {
    const m = r.url().match(/\/prices\/(\w+)/);
    if (m) asked.push(m[1]);
  });
  await open_roster(page, [{ names: ["A"] }, { names: ["B"] }]);
  await roster(page, 2)
    .getByRole("combobox", { name: "Region" })
    .selectOption("euc");
  await expect.poll(() => asked).toContain("euc");
  expect(await bh.roster_regions(page)).toEqual(["nae", "euc"]);
  await persist(page);
  await page.reload();
  expect(await bh.roster_regions(page)).toEqual(["nae", "euc"]);
  await expect(
    roster(page, 2).getByRole("combobox", { name: "Region" }),
  ).toHaveValue("euc");
  await expect(
    roster(page, 1).getByRole("combobox", { name: "Region" }),
  ).toHaveValue("nae");
});

test("an accented name routes and reloads", async ({ page }) => {
  await open_roster(page, [{ names: ["Ëlsa"] }]);
  await page.getByRole("link", { name: "Go to character Ëlsa" }).click();
  await expect(page).toHaveURL(`/chars/NA/${encodeURIComponent("Ëlsa")}/calc`);
  await expect(page.getByRole("textbox", { name: "Char name" })).toHaveValue(
    "Ëlsa",
  );
  await persist(page);
  await page.reload();
  await expect(page.getByRole("textbox", { name: "Char name" })).toHaveValue(
    "Ëlsa",
  );
});

test("the roster page's Pending ilevel equals the calc page's", async ({
  page,
}) => {
  await hf.open(page, "/chars/NA/Newchar/calc");
  await hf.toggle_column(page, "normal", 15);
  const calc = await hf.read_pending_ilevel(page);
  expect(calc).toBe(1665);
  await page.getByRole("link", { name: "Roster setup" }).click();
  await expect(page.getByText("Pending ilevel:")).toBeVisible();
  expect(await hf.read_pending_ilevel(page)).toBe(calc);
});

test("Fetch from lostark.bible, Confirm: cells, ilevel and class change", async ({
  page,
}) => {
  const scrape = await bh.stub_character(page, { Newchar: bh.char_html("t4") });
  await hf.open(page, "/roster-setup");
  const before = await page.locator("body").textContent();
  await page.getByRole("button", { name: "Fetch from lostark.bible" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByText("This you?")).toBeVisible();
  await dialog.getByRole("button", { name: "Confirm" }).click();
  await expect(dialog).toHaveCount(0);
  expect(scrape.requests).toEqual(["NA/Newchar/"]);
  await expect(page.getByText(/Achieved ilevel: (?!1640)/)).toBeVisible();
  expect(before).toContain("Achieved ilevel: 1640");
  await persist(page);
  await page.goto("/chars/NA/Newchar/calc");
  expect(await hf.cell_status(page, "normal", "Helmet", 20)).toBe(
    "FetchedDone",
  );
  expect(await hf.cell_status(page, "normal", "Helmet", 21)).toBe("NotYet");
  await expect(page.getByTitle("Bard").first()).toBeVisible();
  const profile = await bh.profile(page, "Newchar");
  expect(profile.class_name).toBe("Bard");
  expect(profile.fetched_at).toBeGreaterThan(0);
  await page.reload();
  expect(await hf.cell_status(page, "normal", "Helmet", 20)).toBe(
    "FetchedDone",
  );
});

test("Fetch remaining characters: Confirm all adds a row each", async ({
  page,
}) => {
  await bh.stub_character(
    page,
    {
      Beta: bh.char_html("t4", "Berserker"),
      Gamma: bh.char_html("serca", "Bard"),
    },
    { Alpha: ["Alpha", "Beta", "Gamma"] },
  );
  await open_roster(page, [{ names: ["Alpha"] }]);
  await page
    .getByRole("button", { name: "Fetch remaining characters" })
    .click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByText("Are these your children?")).toBeVisible();
  await dialog.getByRole("button", { name: "Confirm all" }).click();
  await expect(dialog).toHaveCount(0);
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Alpha", "Beta", "Gamma"]]);
  expect((await bh.profile(page, "Beta")).class_name).toBe("Berserker");
  expect((await bh.profile(page, "Beta")).tier).toBe(0);
  expect((await bh.profile(page, "Gamma")).tier).toBe(1);
  await persist(page);
  await page.reload();
  expect(await bh.roster_names(page)).toEqual([["Alpha", "Beta", "Gamma"]]);
  expect((await bh.profile(page, "Gamma")).tier).toBe(1);
});

test("roster-bound mats are per roster, on the market page and the calc page", async ({
  page,
}) => {
  await open_roster(page, [{ names: ["A"] }, { names: ["B"] }]);
  await page.goto("/market-mats");
  const tier4 = page.getByRole("group", { name: "T4 materials" });
  const picker = page.getByRole("combobox", { name: "Active roster" });
  const red = () =>
    hf
      .material_row(tier4, "Red")
      .getByRole("textbox", { name: "Roster bound owned" });
  await picker.selectOption({ label: "Roster 2" });
  await hf.set_material(tier4, "Red", "Roster bound owned", 42);
  await expect(red()).toHaveValue("42");
  await picker.selectOption({ label: "Roster 1" });
  await expect(red()).toHaveValue("0");
  await persist(page);
  await page.reload();
  await page
    .getByRole("combobox", { name: "Active roster" })
    .selectOption({ label: "Roster 2" });
  await expect(red()).toHaveValue("42");

  await header_tab(page, "Roster 2").click();
  await page
    .getByRole("combobox", { name: "Chance column" })
    .selectOption({ label: "Roster mats" });
  await expect(
    hf
      .material_row(page, "Red")
      .getByRole("textbox", { name: "Roster bound owned" }),
  ).toHaveValue("42");
  await header_tab(page, "Roster 1").click();
  await page
    .getByRole("combobox", { name: "Chance column" })
    .selectOption({ label: "Roster mats" });
  await expect(
    hf
      .material_row(page, "Red")
      .getByRole("textbox", { name: "Roster bound owned" }),
  ).toHaveValue("0");
});

test("a roster on EU shows EU prices while the other stays on NA", async ({
  page,
}) => {
  await open_roster(page, [{ names: ["A"] }, { names: ["B"] }]);
  await page.goto("/market-mats");
  const tier4 = page.getByRole("group", { name: "T4 materials" });
  const picker = page.getByRole("combobox", { name: "Active roster" });
  const price = () =>
    hf
      .material_row(tier4, "Red")
      .getByRole("textbox", { name: "Market price" })
      .inputValue()
      .then((v) => Number(v.replace(/\D/g, "")));
  await expect.poll(price).toBeGreaterThan(0);
  const na = await price();
  await picker.selectOption({ label: "Roster 2" });
  await page.getByRole("combobox", { name: "Region" }).selectOption("euc");
  const slug = "destiny-crystallized-destruction-stone";
  const ratio =
    market.euc.find((x: any) => x.item_slug === slug).price /
    market.nae.find((x: any) => x.item_slug === slug).price;
  await expect
    .poll(async () => (await price()) / na, { timeout: 15_000 })
    .toBeCloseTo(ratio, 1);
  await picker.selectOption({ label: "Roster 1" });
  expect(await price()).toBe(na);
});

// Names are unique per region only, so the route carries it. Redirects run before the save is
// loaded, so an old link opened cold must still land on the right character.
test("a name in two regions has two urls, and an old link finds its region", async ({
  page,
}) => {
  await open_roster(page, [
    { names: ["Alpha", "Zed"] },
    { names: ["Zed", "Yan"], region: "euc" },
  ]);
  await persist(page);
  expect(await bh.roster_names(page)).toEqual([
    ["Alpha", "Zed"],
    ["Zed", "Yan"],
  ]);
  await roster(page, 2)
    .getByRole("link", { name: "Go to character Zed" })
    .click();
  await expect(page).toHaveURL(/\/chars\/EU\/Zed\/calc$/);
  expect(await bh.with_store(page, (s) => s.active_region)).toBe("euc");

  // old region-less link, opened on a fresh load
  await hf.open(page, "/chars/Yan/scanner");
  await expect(page).toHaveURL(/\/chars\/EU\/Yan\/scanner$/);
  expect(await bh.with_store(page, (s) => s.active_profile.char_name)).toBe(
    "Yan",
  );

  // moving a roster into a region where a name is taken renames it
  await hf.open(page, "/roster-setup");
  await roster(page, 2).getByLabel("Region").selectOption("nae");
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([
      ["Alpha", "Zed"],
      ["Zed1", "Yan"],
    ]);
});
