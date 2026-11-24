import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import * as hf from "./helpers.ts";

let errors: string[];
test.beforeEach(async ({ page }) => {
  test.setTimeout(120_000);
  errors = hf.watch_errors(page);
  await hf.stub_market(page);
});
test.afterEach(() => expect(errors).toEqual([]));

const CALC = "/chars/NA/Newchar/calc";
const BIG = { Red: 1e7, Blue: 1e7, Leaps: 1e7, Fusion: 1e7, Shards: 1e8 };

const forbidden = (page: Page): Promise<string[]> =>
  hf.store_eval(page, (s: any) => [...s.active_profile.forbidden_chests]);
const stored_chests = (page: Page): Promise<string[]> =>
  hf.store_eval(page, (s: any) => s.active_chests.map((c: any) => c.key));

// stored chests, as the scanner would leave them
async function inject_chests(page: Page, chests: object[]) {
  await hf.store_eval(
    page,
    (s: any, list: any[]) => {
      for (const c of list)
        s.active_profile.chests.push({ obtain_all: false, ...c });
    },
    chests,
  );
}

// a roster-bound chest of one material, through the editor, left in edit mode
async function add_chest(page: Page, name: string, count: number, red: number) {
  const edit = page.getByLabel("Edit chests");
  if (!(await edit.isChecked())) await edit.check();
  await page.getByLabel("New chest count").fill(String(count));
  await page.getByLabel("New chest name").fill(name);
  await page
    .getByLabel("New chest ownership")
    .selectOption({ label: "Roster bound" });
  await page.getByLabel("Option 1 material").selectOption("Red");
  await page.getByLabel("Option 1 amount").fill(String(red));
  await page.getByRole("button", { name: "Add chest" }).click();
}

const two_boxes = [
  { key: "t|a", title: "Box A", band: 1, count: 1, options: [BIG] },
  { key: "t|b", title: "Box B", band: 1, count: 1, options: [BIG] },
];

test("forbidding a merged chest stack forbids every stored chest in it", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.toggle_column(page, "normal", 15);
  await hf.wait_for_optimizer(page);
  const base = await hf.read_gold(page);
  await inject_chests(page, two_boxes);
  await hf.wait_for_optimizer(page);
  const stocked = await hf.read_gold(page);
  expect(stocked).toBeLessThan(base);

  // two stored chests, one processed stack
  await page.getByLabel(/Show all chests/).check();
  const forbid = page.getByRole("button", {
    name: "Forbid Box A / Box B on this character",
  });
  const allow = (box: string) =>
    page.getByRole("button", { name: `Allow ${box} on this character` });
  await forbid.click();
  await expect
    .poll(async () => (await forbidden(page)).sort())
    .toEqual(["t|a", "t|b"]);
  // forbidden chests are listed one by one, under their stored titles
  await expect(allow("Box A")).toBeVisible();
  await expect(allow("Box B")).toBeVisible();
  await expect(page.getByText("Forbidden on this character")).toHaveCount(2);
  await hf.wait_for_optimizer(page);
  expect(await hf.read_gold(page)).toBeGreaterThan(stocked);

  // survives a reload
  await hf.reload(page);
  await page.getByLabel(/Show all chests/).check();
  await expect
    .poll(async () => (await forbidden(page)).sort())
    .toEqual(["t|a", "t|b"]);
  await expect(page.getByText("Forbidden on this character")).toHaveCount(2);

  // the forbidden state is by stored key, so it survives a tier change both ways
  await page.getByRole("button", { name: "Convert to T4.5 Serca" }).click();
  await expect(
    page.getByRole("button", { name: "Revert back to T4" }),
  ).toBeVisible();
  await expect
    .poll(async () => (await forbidden(page)).sort())
    .toEqual(["t|a", "t|b"]);
  await expect(page.getByText("Forbidden on this character")).toHaveCount(2);
  await page.getByRole("button", { name: "Revert back to T4" }).click();
  await expect(
    page.getByRole("button", { name: "Convert to T4.5 Serca" }),
  ).toBeVisible();
  await expect(page.getByText("Forbidden on this character")).toHaveCount(2);

  await allow("Box A").click();
  // the list re-lays out; a click before it settles can land on the moved Forbid button
  await expect.poll(() => forbidden(page)).toEqual(["t|b"]);
  await allow("Box B").click();
  await expect.poll(() => forbidden(page)).toEqual([]);
  await expect(forbid).toBeVisible();
  await expect(page.getByText("Forbidden on this character")).toHaveCount(0);
});

test("forbidding in the chest editor shows in the read-only view", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await inject_chests(page, two_boxes);
  await page.getByLabel(/Show all chests/).check();
  await page.getByLabel("Edit chests").check();
  await page
    .getByRole("button", { name: "Forbid Box A on this character" })
    .click();
  await expect.poll(() => forbidden(page)).toEqual(["t|a"]);
  await page.getByLabel("Edit chests").uncheck();
  // Box A is greyed out; Box B stands alone in the allowed stack
  await expect(
    page.getByRole("button", { name: "Allow Box A on this character" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Forbid Box B on this character" }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Forbid Box B on this character" })
    .click();
  await expect
    .poll(async () => (await forbidden(page)).sort())
    .toEqual(["t|a", "t|b"]);
});

test("chest advice splits a two-option chest by the optimizer's state", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await inject_chests(page, [
    {
      key: "t|choice",
      title: "Choice box",
      band: 1,
      count: 20,
      options: [{ Red: 3000 }, { Blue: 3000 }],
    },
  ]);
  await hf.toggle_column(page, "normal", 15);
  await hf.wait_for_optimizer(page);
  const advice = page.getByRole("group", { name: "Chest opening advice" });
  await expect(advice).toContainText(/Open [\d,]+ boxes? as/);
  // the split is of all 20 boxes
  const text = await advice.textContent();
  const counts = [...text.matchAll(/(?:Open|open) ([\d,]+) boxes? as/g)].map(
    (m) => Number(m[1].replaceAll(",", "")),
  );
  expect(counts.reduce((a, b) => a + b, 0)).toBe(20);
});

test("chest counts and deletions persist, and a re-added chest is not forbidden", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await add_chest(page, "Persist box", 2, 100);
  await page.getByLabel(/Show all chests/).check();
  const count = page.getByLabel("Persist box count");
  await count.fill("5");
  await count.blur();
  await hf.toggle_column(page, "normal", 15);
  await hf.wait_for_optimizer(page);
  await expect(count).toHaveValue("5");
  await hf.reload(page);
  await page.getByLabel("Edit chests").check();
  await page.getByLabel(/Show all chests/).check();
  await expect(page.getByLabel("Persist box count")).toHaveValue("5");

  await page
    .getByRole("button", { name: "Forbid Persist box on this character" })
    .click();
  await expect.poll(async () => (await forbidden(page)).length).toBe(1);
  await page.getByRole("button", { name: "Delete Persist box" }).click();
  await expect(page.getByLabel("Persist box count")).toHaveCount(0);
  await expect.poll(() => forbidden(page)).toEqual([]);
  expect(await stored_chests(page)).toEqual([]);

  // the same chest again is a new chest
  await add_chest(page, "Persist box", 2, 100);
  await expect(
    page.getByRole("button", { name: "Forbid Persist box on this character" }),
  ).toBeVisible();
  await expect.poll(() => forbidden(page)).toEqual([]);
});

const num = (text: string) => Number(text.replaceAll(",", ""));
const owned_box = (page: Page, mat = "Red", name = "Bound owned") =>
  hf.material_row(page, mat).getByRole("textbox", { name });
const owned = async (page: Page, mat = "Red") =>
  num(await owned_box(page, mat).inputValue());
const first_row = (page: Page) =>
  page
    .locator(".instruction-row")
    .filter({ has: page.getByText("1st", { exact: true }) })
    .first();
const row_name = async (row: ReturnType<typeof first_row>) =>
  (await row.getAttribute("aria-label"))!;
const level_of = (name: string) => Number(name.match(/\+(\d+)$/)![1]);
const piece_of = (name: string) => name.split(" ")[0];

test("tier conversion remaps the grids, keeps materials and reverts", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.set_material(page, "Red", "Bound owned", 12345);
  await hf.toggle_column(page, "normal", 22);
  await hf.cell(page, "normal", "Helmet", 20).click();
  await hf.wait_for_optimizer(page);
  expect(await hf.cell_status(page, "normal", "Helmet", 20)).toBe("Done");
  expect(await hf.cell_status(page, "normal", "Weapon", 22)).toBe("Want");

  await page.getByRole("button", { name: "Convert to T4.5 Serca" }).click();
  await expect(
    page.getByRole("button", { name: "Revert back to T4" }),
  ).toBeVisible();
  // T4 +20..+22 are Serca +11..+13
  for (const piece of ["Helmet", "Weapon"]) {
    expect(await hf.cell_status(page, "normal", piece, 12)).toBe("Want");
    expect(await hf.cell_status(page, "normal", piece, 13)).toBe("Want");
    expect(await hf.cell_status(page, "normal", piece, 14)).toBe("NotYet");
  }
  await expect(hf.grid(page, "adv")).toHaveCount(0);
  // the T4 stock stays as typed, counted as Serca by the payload
  expect(
    await hf.store_eval(
      page,
      (s: any) => s.active_profile.bound_budgets.values.Red,
    ),
  ).toBe("12,345");

  await hf.reload(page);
  await expect(
    page.getByRole("button", { name: "Revert back to T4" }),
  ).toBeVisible();
  expect(await hf.cell_status(page, "normal", "Helmet", 12)).toBe("Want");

  await page.getByRole("button", { name: "Revert back to T4" }).click();
  await expect(
    page.getByRole("button", { name: "Convert to T4.5 Serca" }),
  ).toBeVisible();
  // ticked by hand, so Done again, not the Serca floor's FetchedDone
  expect(await hf.cell_status(page, "normal", "Helmet", 20)).toBe("Done");
  expect(await hf.cell_status(page, "normal", "Helmet", 15)).toBe("Done");
  expect(await hf.cell_status(page, "normal", "Helmet", 21)).toBe("Want");
  expect(await hf.cell_status(page, "normal", "Helmet", 22)).toBe("Want");
  expect(await hf.cell_status(page, "normal", "Helmet", 23)).toBe("NotYet");
});

test("revert is disabled while a Serca level has no T4 equal", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await page.getByRole("button", { name: "Convert to T4.5 Serca" }).click();
  await hf.toggle_column(page, "normal", 15);
  const revert = page.getByRole("button", { name: "Revert back to T4" });
  await expect(revert).toBeDisabled();
  await revert.hover({ force: true });
  await expect(page.getByText(/cannot be mapped directly/)).toBeVisible();
  // clearing the column makes it possible again
  await hf.toggle_column(page, "normal", 15);
  await hf.toggle_column(page, "normal", 15);
  await expect(revert).toBeEnabled();
});

test("slider deducts owned materials, Succeed finishes the upgrade", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.set_material(page, "Red", "Bound owned", 1_000_000);
  await hf.cell(page, "normal", "Weapon", 12).click();
  await hf.wait_for_optimizer(page);
  const row = first_row(page);
  const name = await row_name(row);
  const before = await owned(page);

  // slider with auto-deduct off leaves the owned mats alone
  const deduct = page.getByLabel("Slider auto-deducts costs");
  await deduct.uncheck();
  await row.getByRole("slider", { name: "Taps slider" }).fill("2");
  expect(await owned(page)).toBe(before);
  await row.getByRole("slider", { name: "Taps slider" }).fill("0");

  await deduct.check();
  await row.getByRole("slider", { name: "Taps slider" }).fill("2");
  await expect.poll(() => owned(page)).toBeLessThan(before);
  const spent = await owned(page);
  await expect(
    row.getByRole("spinbutton", { name: "Taps since last run" }),
  ).toHaveValue("2");

  await row.getByRole("button", { name: "Succeed" }).click();
  expect(
    await hf.cell_status(page, "normal", piece_of(name), level_of(name)),
  ).toBe("Done");
  await hf.wait_for_optimizer(page);
  await expect(hf.upgrade_row(page, name)).toHaveCount(0);
  // the other upgrade is first now
  await expect(first_row(page)).toBeVisible();
  expect(await row_name(first_row(page))).not.toBe(name);
  const after = await owned(page);
  expect(after).toBeLessThanOrEqual(spent);

  await hf.reload(page);
  expect(
    await hf.cell_status(page, "normal", piece_of(name), level_of(name)),
  ).toBe("Done");
  expect(await owned(page)).toBe(after);
});

test("manual artisan and base chance persist, Confirm reruns, Reset restores", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.cell(page, "normal", "Helmet", 12).click();
  await hf.wait_for_optimizer(page);
  const row = first_row(page);
  const name = await row_name(row);
  const artisan = row.getByRole("textbox", { name: "Current artisan energy" });
  const chance = row.getByRole("textbox", { name: "Current base chance" });

  await artisan.fill("25");
  await artisan.blur();
  await expect(
    row.getByRole("button", { name: "Restore default" }),
  ).toBeVisible();
  const default_chance = parseFloat(await chance.inputValue());
  await chance.fill(String(default_chance * 1.5));
  await chance.blur();
  const snapped = await chance.inputValue();
  expect(snapped).not.toBe(String(default_chance));

  // Confirm restarts the optimizer and leaves the inputs as set
  await row.getByRole("button", { name: "Confirm" }).click();
  await hf.wait_for_optimizer(page);
  const same = hf.upgrade_row(page, name);
  await expect(
    same.getByRole("textbox", { name: "Current artisan energy" }),
  ).toHaveValue(/^25(\.00)?$/);

  await hf.reload(page);
  await expect(
    same.getByRole("textbox", { name: "Current artisan energy" }),
  ).toHaveValue(/^25(\.00)?$/);
  await expect(
    same.getByRole("textbox", { name: "Current base chance" }),
  ).toHaveValue(snapped);

  await same.getByRole("button", { name: "Reset this upgrade" }).click();
  await hf.wait_for_optimizer(page);
  await expect(
    same.getByRole("textbox", { name: "Current artisan energy" }),
  ).toHaveValue(/^0(\.00)?$/);
  await expect(
    same.getByRole("textbox", { name: "Current base chance" }),
  ).toHaveValue(/^10(\.00)?$/);
});

test("free taps: special leaps make a free-tap row, Succeed and failing spend them", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.toggle_column(page, "normal", 15);
  const leaps_box = page.getByRole("textbox", { name: "Special leaps owned" });
  await leaps_box.fill("1000");
  await leaps_box.blur();
  await hf.wait_for_optimizer(page);
  const free = () =>
    page
      .locator(".instruction-row")
      .filter({ has: page.getByLabel("Special leaps remaining") })
      .first();
  await expect(
    page.getByText("Free tap this until you run out or succeed").first(),
  ).toBeVisible();
  const row = free();
  const name = await row_name(row);
  const leaps = page.getByRole("textbox", { name: "Special leaps owned" });

  await row.getByLabel("Special leaps remaining").fill("900");
  await row.getByRole("button", { name: "Succeed" }).click();
  expect(
    await hf.cell_status(page, "normal", piece_of(name), level_of(name)),
  ).toBe("Done");
  await expect(leaps).toHaveValue("900");

  await hf.wait_for_optimizer(page);
  await free().getByRole("button", { name: "All free taps failed" }).click();
  await expect(leaps).toHaveValue("0");
  await hf.wait_for_optimizer(page);
  await expect(
    page.getByText("Free tap this until you run out or succeed"),
  ).toHaveCount(0);
});

test("advanced honing details persist and Succeed marks the matching cell", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.toggle_column(page, "adv", 40);
  await hf.wait_for_optimizer(page);
  const row = hf.upgrade_row(page, "Helmet Advanced +10 - 20");
  const field = (r: typeof row, name: string) =>
    r.getByRole("spinbutton", { name });
  const expand = () =>
    page.getByRole("button", { name: /^Show all \d+ upgrades$/ }).click();
  await expand();

  await field(row, "Current upgrade").fill("15");
  await field(row, "Current upgrade").blur();
  await field(row, "Current xp").fill("30");
  await field(row, "Current xp").blur();
  // a Chisel is offered once past the first level of the range with no grace
  await row.getByRole("checkbox", { name: "Chisel" }).check();
  await hf.reload(page);
  await expand();
  const again = hf.upgrade_row(page, "Helmet Advanced +10 - 20");
  await expect(field(again, "Current upgrade")).toHaveValue("15");
  await expect(field(again, "Current xp")).toHaveValue("30");
  await expect(again.getByRole("checkbox", { name: "Chisel" })).toBeChecked();

  // grace progress hides the Chisel
  await field(again, "Grace progress").fill("2");
  await field(again, "Grace progress").blur();
  await expect(again.getByRole("checkbox", { name: "Chisel" })).toHaveCount(0);
  await hf.reload(page);
  await expand();
  await expect(
    field(hf.upgrade_row(page, "Helmet Advanced +10 - 20"), "Grace progress"),
  ).toHaveValue("2");

  // Naber's Awl at full grace from the +20 - 30 range on
  const awl_row = hf.upgrade_row(page, "Helmet Advanced +20 - 30");
  await field(awl_row, "Grace progress").fill("6");
  await field(awl_row, "Grace progress").blur();
  await awl_row.getByRole("checkbox", { name: "Naber's Awl" }).check();
  await hf.reload(page);
  await expand();
  await expect(
    hf
      .upgrade_row(page, "Helmet Advanced +20 - 30")
      .getByRole("checkbox", { name: "Naber's Awl" }),
  ).toBeChecked();

  // Succeed on the +10 - 20 range is the +20 cell
  expect(await hf.cell_status(page, "adv", "Helmet", 20)).toBe("Want");
  await hf
    .upgrade_row(page, "Helmet Advanced +10 - 20")
    .getByRole("button", { name: "Succeed" })
    .click();
  expect(await hf.cell_status(page, "adv", "Helmet", 20)).toBe("Done");
  expect(await hf.cell_status(page, "adv", "Helmet", 30)).toBe("Want");
});

test("the owned tickbox disables the input and the material", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.toggle_column(page, "normal", 15);
  await hf.set_material(page, "Red", "Bound owned", 1000);
  await hf.wait_for_optimizer(page);
  const owned_gold = await hf.read_gold(page);

  const tick = hf
    .material_row(page, "Red")
    .getByRole("checkbox", { name: "Bound owned enabled" });
  await tick.uncheck();
  await expect(owned_box(page)).toBeDisabled();
  await expect.poll(() => hf.read_gold(page)).not.toBe(owned_gold);

  await hf.reload(page);
  await expect(tick).not.toBeChecked();
  await expect(owned_box(page)).toBeDisabled();
});

test("the chance column select picks which owned column shows, per character", async ({
  page,
}) => {
  await hf.open(page, CALC);
  const select = page.getByLabel("Chance column");
  await select.selectOption({ label: "Roster mats" });
  await hf.set_material(page, "Red", "Roster bound owned", 777);
  await select.selectOption({ label: "Tradable" });
  await hf.set_material(page, "Red", "Tradable owned", 555);
  await select.selectOption({ label: "Bound mats" });
  await expect(owned_box(page)).toHaveValue("0");

  // a second character of the roster sees the roster-bound stock
  const name: string = await hf.store_eval(page, (s: any) => {
    const profile = s.add_char(s.active_profile.roster_id);
    return s.char_label(profile);
  });
  await hf.reload(page);
  await hf.open(page, `/chars/NA/${encodeURIComponent(name)}/calc`);
  const other = page.getByLabel("Chance column");
  await other.selectOption({ label: "Roster mats" });
  await expect(owned_box(page, "Red", "Roster bound owned")).toHaveValue("777");
  await other.selectOption({ label: "Tradable" });
  await expect(owned_box(page, "Red", "Tradable owned")).toHaveValue("555");

  // each character keeps its own selection
  await hf.open(page, CALC);
  await expect(page.getByLabel("Chance column")).toHaveValue("0");
  await page.getByLabel("Chance column").selectOption({ label: "Roster mats" });
  await hf.reload(page);
  await expect(page.getByLabel("Chance column")).toHaveValue("1");
  await expect(owned_box(page, "Red", "Roster bound owned")).toHaveValue("777");
});

test("leftover bands: presets, per-band selects and Custom", async ({
  page,
}) => {
  await hf.open(page, CALC);
  const treat = page.getByRole("group", { name: "Treat as", exact: true });
  const treating = page.getByRole("group", {
    name: "Treating as",
    exact: true,
  });
  const pressed = (g: typeof treat, name: string) =>
    g.getByRole("button", { name, exact: true });
  await expect(pressed(treat, "Main")).toHaveAttribute("aria-pressed", "true");
  await expect(pressed(treating, "Main")).toHaveAttribute(
    "aria-pressed",
    "true",
  );

  await pressed(treat, "Alt").click();
  await expect(pressed(treating, "Alt")).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await expect(pressed(treating, "Main")).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  await page.getByRole("button", { name: "Customise" }).click();
  await expect(page.getByLabel("Roster leftover value")).toHaveValue(
    "TaxedSell",
  );

  await page
    .getByLabel("Tradable leftover value")
    .selectOption({ label: "market" });
  await expect(treat.getByText("Custom")).toBeVisible();
  await expect(treating.getByText("Custom")).toBeVisible();
  await expect(pressed(treat, "Alt")).toHaveAttribute("aria-pressed", "false");
  await hf.reload(page);
  await page.getByRole("button", { name: "Customise" }).click();
  await expect(page.getByLabel("Tradable leftover value")).toHaveValue(
    "Market",
  );

  await pressed(treating, "Main").click();
  await expect(page.getByLabel("Roster leftover value")).toHaveValue(
    "Worthless",
  );
  await expect(page.getByLabel("Tradable leftover value")).toHaveValue(
    "TaxedSell",
  );
  await expect(treat.getByText("Custom")).toHaveCount(0);
  await expect(treating.getByText("Custom")).toHaveCount(0);
  await expect(pressed(treat, "Main")).toHaveAttribute("aria-pressed", "true");
});

test("show all mats, cumulative graph and special leaps inputs", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.toggle_column(page, "normal", 15);
  await hf.wait_for_optimizer(page);
  const shown = (await hf.material_graphs(page)).length;
  await page.getByLabel("Show all mats").check();
  await expect
    .poll(async () => (await hf.material_graphs(page)).length)
    .toBeGreaterThan(shown);

  const cumulative = page.getByLabel("Cumulative graph");
  await cumulative.check();
  const leaps = page.getByRole("textbox", { name: "Special leaps owned" });
  await leaps.fill("12");
  await leaps.blur();
  await hf.reload(page);
  await expect(cumulative).toBeChecked();
  await expect(page.getByLabel("Show all mats")).toBeChecked();
  await expect(leaps).toHaveValue("12");

  // a Serca character also lists the T4 stock that converts into each row
  await page.getByLabel("Show all mats").uncheck();
  await page.getByRole("button", { name: "Convert to T4.5 Serca" }).click();
  await expect(hf.material_row(page, "Red")).toHaveCount(0);
  await expect(
    page.getByRole("textbox", { name: "Lower tier special leaps owned" }),
  ).toHaveCount(0);
  await page.getByLabel("Show all mats").check();
  await expect(hf.material_row(page, "Red")).toHaveCount(1);
  const lower = page.getByRole("textbox", {
    name: "Lower tier special leaps owned",
  });
  await lower.fill("40");
  await lower.blur();
  await expect(lower).toHaveValue("40");
  await hf.reload(page);
  await expect(
    page.getByRole("textbox", { name: "Lower tier special leaps owned" }),
  ).toHaveValue("40");
});

test("Reset this char: Cancel keeps the data, Reset clears it", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.toggle_column(page, "normal", 15);
  await hf.set_material(page, "Red", "Bound owned", 999);
  await hf.wait_for_optimizer(page);
  expect(await hf.read_pending_ilevel(page)).toBe(1665);

  await page.getByRole("button", { name: "Reset this char" }).click();
  await page.getByRole("button", { name: "Cancel" }).click();
  expect(await hf.cell_status(page, "normal", "Helmet", 15)).toBe("Want");

  await page.getByRole("button", { name: "Reset this char" }).click();
  await page.getByRole("button", { name: "Reset", exact: true }).click();
  expect(await hf.cell_status(page, "normal", "Helmet", 15)).toBe("NotYet");
  await expect(owned_box(page)).toHaveValue("0");
  await expect.poll(() => hf.read_pending_ilevel(page)).toBe(1640);
  await hf.reload(page);
  expect(await hf.cell_status(page, "normal", "Helmet", 15)).toBe("NotYet");
  expect(await hf.read_pending_ilevel(page)).toBe(1640);
});

test("optimizer panel: auto start, force x10, simple strategy comparison", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.toggle_column(page, "normal", 15);
  await hf.wait_for_optimizer(page);
  const base = await hf.read_gold(page);

  // with auto start off a change does not start an optimizer run
  const runs = () =>
    page.evaluate(async () => {
      const url = performance
        .getEntriesByType("resource")
        .map((entry) => entry.name)
        .filter((name) => name.includes("Stores/RuntimeState.ts"))
        .pop();
      const { useRuntimeStore } = await import(url);
      return useRuntimeStore().optimizer.run_counter as number;
    });
  const auto = page.getByLabel(/Auto start optimizer/);
  await auto.uncheck();
  await expect(
    page.getByText("Optimizer will not re-run on updates"),
  ).toBeVisible();
  await page.waitForTimeout(1500);
  const before = await runs();
  await hf.cell(page, "normal", "Helmet", 16).click();
  await page.waitForTimeout(1500);
  expect(await runs()).toBe(before);
  await hf.reload(page);
  await expect(auto).not.toBeChecked();
  await page.waitForTimeout(1500);
  const reloaded = await runs();
  await auto.check();
  await hf.wait_for_optimizer(page);
  expect(await runs()).toBeGreaterThan(reloaded);

  // force x10 is a stored setting that restarts the run
  const x10 = page.getByLabel("Force x10 streak length");
  await expect(x10).toBeChecked();
  await x10.uncheck();
  await hf.wait_for_optimizer(page);
  expect(
    await hf.store_eval(page, (s: any) => s.active_profile.min_resolution),
  ).toBe(1);
  await x10.check();
  await hf.wait_for_optimizer(page);
  expect(
    await hf.store_eval(page, (s: any) => s.active_profile.min_resolution),
  ).toBe(10);

  // a simple strategy is no better than the optimizer
  const optimal = await hf.read_gold(page);
  expect(optimal).toBeGreaterThan(0);
  await page
    .getByRole("button", { name: "Compare with simple strategies" })
    .click();
  const juice = page.getByRole("group", { name: "Juice Usage", exact: true });
  await juice.getByRole("radio", { name: "Full juice" }).click();
  await expect(page.getByText("(Not optimized)").first()).toBeVisible();
  await expect(
    page.getByText("FOR COMPARISON PURPOSE ONLY").first(),
  ).toBeVisible();
  await expect
    .poll(() => hf.read_gold(page))
    .toBeGreaterThanOrEqual(optimal * 0.98);
  expect(base).toBeGreaterThan(0);

  // closing the comparison goes back to the optimizer's own instructions
  await page
    .getByRole("button", { name: "Compare with simple strategies" })
    .click();
  await expect(page.getByText("(Not optimized)")).toHaveCount(0);
});

test("free-tap hint can be dismissed for good, long lists expand", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.toggle_column(page, "normal", 15);
  const leaps = page.getByRole("textbox", { name: "Special leaps owned" });
  await leaps.fill("1000");
  await leaps.blur();
  await hf.wait_for_optimizer(page);
  await expect(page.getByText("free taps failed").first()).toBeVisible();
  await page.getByRole("button", { name: "OK", exact: true }).first().click();
  await expect(page.getByText("Only if all")).toHaveCount(0);
  await hf.reload(page);
  await hf.wait_for_optimizer(page);
  await expect(
    page.getByText("Free tap this until you run out or succeed").first(),
  ).toBeVisible();
  await expect(page.getByText("Only if all")).toHaveCount(0);

  const rows = page.locator(".instruction-row");
  await expect(rows).toHaveCount(10);
  await page.getByRole("button", { name: /^Show all \d+ upgrades$/ }).click();
  await expect(rows).toHaveCount(30);
  await page.getByRole("button", { name: "Collapse" }).click();
  await expect(rows).toHaveCount(10);
});

test("column headers on Advanced and the Serca vambrace row set cells", async ({
  page,
}) => {
  await hf.open(page, CALC);
  await hf.toggle_column(page, "adv", 20);
  for (const [level, status] of [
    [10, "Want"],
    [20, "Want"],
    [30, "NotYet"],
  ] as const)
    expect(await hf.cell_status(page, "adv", "Helmet", level)).toBe(status);
  expect(await hf.cell_status(page, "adv", "Weapon", 20)).toBe("Want");
  await hf.reload(page);
  expect(await hf.cell_status(page, "adv", "Weapon", 20)).toBe("Want");

  await page.getByRole("button", { name: "Convert to T4.5 Serca" }).click();
  await hf.cell(page, "normal", "Vambrace", 3).click();
  expect(await hf.cell_status(page, "normal", "Vambrace", 2)).toBe("Want");
  expect(await hf.cell_status(page, "normal", "Vambrace", 4)).toBe("NotYet");
  await hf.cell(page, "normal", "Vambrace", 3).click();
  expect(await hf.cell_status(page, "normal", "Vambrace", 3)).toBe("Done");
  await hf.reload(page);
  expect(await hf.cell_status(page, "normal", "Vambrace", 3)).toBe("Done");
});
