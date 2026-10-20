import { describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
import "./helpers";
import { mount_app } from "../mount";
import TickboxGrid from "@/Components/Character/StatusInput/TickboxGrid.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { NORMAL_COLS } from "@/Utils/Constants";

const { Done, Want, NotYet, FetchedDone } = UpgradeStatus;

async function setup(grid_type: "normal" | "adv" = "normal", tier = 0) {
  const { wrapper } = await mount_app(TickboxGrid, {
    props: { grid_type },
  });
  const store = useRosterStore();
  store.active_profile.tier = tier;
  await wrapper.vm.$nextTick();
  const profile = store.active_profile;
  const grid = () =>
    grid_type === "normal" ? profile.normal_grid : profile.adv_grid;
  const cell = (name: string) =>
    wrapper
      .findAll("button")
      .find((b) => b.attributes("aria-label")?.startsWith(`${name}:`))!;
  const header = (text: string) =>
    wrapper.findAll(".cell-grid button").find((b) => b.text() === text)!;
  return { wrapper, profile, grid, cell, header };
}

// row of a grid as the letters D W N F
const row_text = (row: UpgradeStatus[]) => row.map((s) => "DWNF"[s]).join("");

describe("TickboxGrid", () => {
  it("shows +11..+25 for T4 and +12..+25 for Serca normal honing", async () => {
    const t4 = await setup("normal", 0);
    expect(
      t4.wrapper.findAll(".cell-grid button").map((b) => b.text())[0],
    ).toBe("+11");
    expect(t4.wrapper.findAll(".cell-grid button")).toHaveLength(15);
    const serca = await setup("normal", 1);
    expect(serca.wrapper.findAll(".cell-grid button")).toHaveLength(14);
    // Serca's extra 7th row offers every level
    expect(
      serca.wrapper
        .findAll("button")
        .filter((b) => b.attributes("aria-label")?.startsWith("Vambrace")),
    ).toHaveLength(NORMAL_COLS);
  });

  it("a NotYet click wants every level up to it, keeping locked fetched ones", async () => {
    const { cell, grid } = await setup();
    await cell("Helmet +13").trigger("click");
    expect(row_text(grid()[0])).toBe("FFFFFFFFFF" + "WWW" + "N".repeat(12));
    expect(cell("Helmet +13").attributes("aria-label")).toBe(
      "Helmet +13: Want",
    );
    // the last wanted cell carries its level
    expect(cell("Helmet +13").text()).toBe("+ 13");
    expect(cell("Helmet +12").text()).toBe("");
  });

  it("a Want click marks it and below done; a Done click clears it and above", async () => {
    const { cell, grid } = await setup();
    await cell("Glove +14").trigger("click");
    await cell("Glove +12").trigger("click");
    expect(row_text(grid()[4])).toBe("FFFFFFFFFF" + "DDWW" + "N".repeat(11));
    expect(cell("Glove +12").text()).toBe("✓");
    await cell("Glove +11").trigger("click");
    expect(row_text(grid()[4])).toBe("FFFFFFFFFF" + "N".repeat(15));
  });

  it("a NotYet click below a done level turns the done ones above it to Want", async () => {
    const { cell, grid } = await setup();
    await cell("Chest +15").trigger("click");
    await cell("Chest +15").trigger("click"); // all Done to +15
    await cell("Chest +13").trigger("click"); // +13.. NotYet
    expect(row_text(grid()[2]).slice(10, 16)).toBe("DDNNNN");
    grid()[2][15] = Done; // a stray done above
    await cell("Chest +13").trigger("click");
    expect(row_text(grid()[2]).slice(10, 17)).toBe("DDWNNWN");
  });

  it("locked fetched cells ignore clicks until unlocked", async () => {
    const { cell, grid, profile, wrapper } = await setup("adv");
    grid()[0][0] = FetchedDone;
    await wrapper.vm.$nextTick();
    expect(cell("Helmet +10").classes()).toContain("FetchedDone");
    expect(cell("Helmet +10").attributes("style")).toContain("not-allowed");
    await cell("Helmet +10").trigger("click");
    expect(grid()[0][0]).toBe(FetchedDone);
    profile.lock_fetched_done = false;
    await wrapper.vm.$nextTick();
    expect(cell("Helmet +10").classes()).toContain("Done");
    await cell("Helmet +10").trigger("click");
    expect(row_text(grid()[0])).toBe("NNNN");
  });

  it("a Done click above locked fetched cells leaves them done", async () => {
    const { cell, grid } = await setup();
    await cell("Pants +12").trigger("click");
    await cell("Pants +12").trigger("click");
    await cell("Pants +11").trigger("click");
    expect(row_text(grid()[3]).slice(0, 12)).toBe("FFFFFFFFFFNN");
  });

  it("a header click applies the column's common state to every row but the 7th", async () => {
    const { header, grid, profile, wrapper } = await setup();
    await header("+12").trigger("click");
    for (let row = 0; row < 6; row++)
      expect(row_text(grid()[row]).slice(10, 13)).toBe("WWN");
    expect(row_text(grid()[6])).toBe("N".repeat(NORMAL_COLS));
    expect(header("+12").classes()).toContain("Want");
    await header("+12").trigger("click");
    expect(header("+12").classes()).toContain("Done");
    // every row's keyed upgrade is regenerated from the grid
    expect(Object.keys(profile.keyed_upgrades)).toHaveLength(0);
    await header("+11").trigger("click");
    expect(header("+12").classes()).not.toContain("Done");
    await wrapper.vm.$nextTick();
  });

  it("a header click over fully fetched locked cells does nothing", async () => {
    const { header, grid } = await setup("adv");
    for (const row of grid()) row[0] = FetchedDone;
    await header("+10").trigger("click");
    expect(grid().every((row) => row[0] === FetchedDone)).toBe(true);
    expect(header("+10").classes()).toContain("FetchedDone");
  });

  it("a header click to Want does not undo rows already done there", async () => {
    const { cell, header, grid } = await setup();
    await cell("Helmet +14").trigger("click");
    await cell("Helmet +14").trigger("click"); // Helmet done to +14
    await header("+12").trigger("click"); // the rest are NotYet: column is mixed
    expect(row_text(grid()[0]).slice(10, 14)).toBe("DDDD");
    expect(row_text(grid()[1]).slice(10, 13)).toBe("WWN");
  });

  it("clicks rebuild keyed upgrades and restart the workers", async () => {
    const { cell, profile } = await setup();
    const runtime = useRuntimeStore();
    const hist = vi.spyOn(runtime.histogram, "throttled_start");
    await cell("Weapon +12").trigger("click");
    expect(Object.keys(profile.keyed_upgrades).sort()).toEqual([
      "5,10,true,0",
      "5,11,true,0",
    ]);
    expect(hist).toHaveBeenCalled();
    expect(profile.normal_grid[5][11]).toBe(Want);
    expect(profile.normal_grid[5][12]).toBe(NotYet);
  });
});
