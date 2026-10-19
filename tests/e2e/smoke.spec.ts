import { expect, test } from "@playwright/test";
import * as hf from "./helpers.ts";

let errors: string[];
test.beforeEach(async ({ page }) => {
  errors = hf.watch_errors(page);
  await hf.stub_market(page);
});
test.afterEach(() => expect(errors).toEqual([]));

test("selecting upgrades runs the optimizer", async ({ page }) => {
  await hf.open(page);
  await expect(page).toHaveURL(/\/calc$/);
  expect(await hf.read_gold(page)).toBe(0);

  await hf.toggle_column(page, "normal", 15);
  await hf.wait_for_optimizer(page);

  expect(await hf.cell_status(page, "normal", "Helmet", 15)).toBe("Want");
  expect(await hf.cell_status(page, "normal", "Helmet", 16)).toBe("NotYet");
  expect(await hf.read_pending_ilevel(page)).toBe(1665);
  const gold = await hf.read_gold(page);
  expect(gold).toBeGreaterThan(0);

  // Rust keys every per-material result by label. Indexing one of those by row still
  // type-checks and yields undefined, which surfaces only as an empty graph.
  const graphs = await hf.material_graphs(page);
  expect(graphs.length).toBeGreaterThan(5);
  expect(graphs.filter((g) => !g.drawn).map((g) => g.label)).toEqual([]);

  // owning mats makes it cheaper
  await hf.set_material(page, "Red", "Bound owned", 1_000_000);
  await hf.wait_for_optimizer(page);
  expect(await hf.read_gold(page)).toBeLessThan(gold);

  // state survives a reload
  await page.waitForTimeout(1000);
  await page.reload();
  expect(await hf.cell_status(page, "normal", "Helmet", 15)).toBe("Want");
});

test("single cell cycles NotYet -> Want -> Done", async ({ page }) => {
  await hf.open(page);
  const weapon = hf.cell(page, "normal", "Weapon", 12);
  await weapon.click();
  expect(await hf.cell_status(page, "normal", "Weapon", 11)).toBe("Want");
  expect(await hf.cell_status(page, "normal", "Weapon", 12)).toBe("Want");
  await weapon.click();
  expect(await hf.cell_status(page, "normal", "Weapon", 12)).toBe("Done");
});

for (const route of [
  "/roster-setup",
  "/market-mats",
  "/Newchar/guide",
  "/change-logs",
]) {
  test(`${route} renders without errors`, async ({ page }) => {
    await hf.open(page, route);
  });
}
