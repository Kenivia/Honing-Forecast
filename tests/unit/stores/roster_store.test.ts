import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  new_char_profile,
  DEFAULT_CHAR_PROFILE,
} from "@/Stores/CharacterProfile";
import { write_state } from "@/Stores/ConfigStorage";
import { start_fetch } from "@/Utils/MarketDataFetcher";
import { bundle_key } from "@/Utils/Constants";
import { chest, rich_config } from "./fixtures";

vi.mock("@/Utils/MarketDataFetcher", async (original) => ({
  ...(await original<any>()),
  start_fetch: vi.fn(),
}));

let store: ReturnType<typeof useRosterStore>;
beforeEach(() => {
  setActivePinia(createPinia());
  store = useRosterStore();
  store.replace_config(rich_config());
});

describe("getters", () => {
  it("active profile, roster columns and region follow the active index", () => {
    expect(store.active_profile.char_name).toBe("Gamma");
    expect(store.active_region).toBe("euc");
    expect(store.active_tradable_mats_owned.values.Blue).toBe("77");
    expect(store.active_mats_prices).toBe(store.roster_config.mats_prices.euc);
    store.switch_profile(0);
    expect(store.active_profile.char_name).toBe("Alpha");
    expect(store.active_region).toBe("nae");
    expect(store.active_roster_mats_owned.values.Red).toBe("1,234");
  });

  it("active chests are the character's then its roster's", () => {
    store.switch_profile(0);
    expect(store.active_chests.map((c) => c.title)).toEqual([
      "Alpha chest",
      "Alpha chest 2",
      "Roster chest",
    ]);
    store.switch_profile(1);
    expect(store.active_chests.map((c) => c.title)).toEqual(["Roster chest"]);
  });

  it("a roster without a chest entry gives the character's chests only", () => {
    store.roster_config.roster_chests = {};
    store.switch_profile(0);
    expect(store.active_chests).toHaveLength(2);
  });

  it("processed chests follow the character's tier", () => {
    store.roster_config.profiles[1].chests = [chest("Serca", 0)];
    store.roster_config.roster_chests = {};
    store.switch_profile(1); // tier 1
    const labels = store.processed_chests.flatMap((c) =>
      c.options.flatMap((o) => Object.keys(o)),
    );
    expect(labels).not.toContain("Red");
    expect(labels).toContain("Serca Red");
  });

  it("roster ids are unique and sorted", () => {
    store.roster_config.profiles[0].roster_id = 5;
    expect(store.roster_ids).toEqual([0, 1, 5]);
  });

  it("unit prices use the selected bundle", () => {
    store.switch_profile(0);
    const prices = store.roster_config.mats_prices.nae;
    prices.values[bundle_key("Shards", 1000)] = "100";
    prices.values[bundle_key("Shards", 3000)] = "600";
    store.roster_config.selected_bundles.nae.Shards = 1000;
    expect(store.active_unit_prices.Shards).toBeCloseTo(0.1);
    store.roster_config.selected_bundles.nae.Shards = 3000;
    expect(store.active_unit_prices.Shards).toBeCloseTo(0.2);
  });

  it("effective price is never above the unit price", () => {
    for (const [label, price] of Object.entries(store.active_unit_prices))
      expect(store.active_effective_prices[label]).toBeLessThanOrEqual(price);
  });
});

describe("actions", () => {
  it("init loads what is in localStorage", () => {
    write_state({ roster_config: rich_config() });
    setActivePinia(createPinia());
    const fresh = useRosterStore();
    expect(fresh.all_profiles).toHaveLength(1);
    fresh.init();
    expect(fresh.all_profiles.map((p) => p.char_name)).toEqual([
      "Alpha",
      "Beta",
      "Gamma",
    ]);
  });

  it("add_profile appends as given", () => {
    const p = { ...new_char_profile(), char_name: "Delta", roster_id: 1 };
    store.add_profile(p);
    expect(store.all_profiles.at(-1).char_name).toBe("Delta");
    expect(store.all_profiles.at(-1).id).toBe(p.id);
  });

  it("reset keeps id, name and roster, clears everything else", () => {
    store.switch_profile(0);
    const { id } = store.active_profile;
    store.active_profile.tier = 1;
    store.reset_active_profile();
    const p = store.active_profile;
    expect([p.id, p.char_name, p.roster_id]).toEqual([id, "Alpha", 0]);
    expect(p.tier).toBe(0);
    expect(p.chests).toEqual([]);
    expect(p.income_manual).toBe(false);
    expect(p.bound_income).toEqual({});
    expect(p.choice_levels).toEqual({});
    expect(p.normal_grid).toEqual(DEFAULT_CHAR_PROFILE.normal_grid);
    // other profiles untouched
    expect(store.all_profiles[2].bound_income).toEqual({ Blue: 9 });
  });

  it("region change sets the active roster's region and fetches it", () => {
    store.switch_profile(0);
    store.active_region_change({ target: { value: "euc" } } as any);
    expect(store.roster_config.all_regions).toEqual({ 0: "euc", 1: "euc" });
    expect(start_fetch).toHaveBeenCalledWith("euc");
  });
});

describe("new_char_profile", () => {
  it("gives a fresh id and a deep copy of the defaults", () => {
    const a = new_char_profile();
    const b = new_char_profile();
    expect(a.id).not.toBe(b.id);
    a.normal_grid[0][0] = 9 as any;
    a.bound_budgets.values.Red = "5";
    expect(b.normal_grid[0][0]).toBe(DEFAULT_CHAR_PROFILE.normal_grid[0][0]);
    expect(DEFAULT_CHAR_PROFILE.bound_budgets.values.Red).toBe("0");
    expect(DEFAULT_CHAR_PROFILE.id).toBe("");
  });
});

describe("roster CRUD", () => {
  const names = () => store.all_profiles.map((p) => p.char_name);

  it("deleting a character keeps the active one by id", () => {
    store.delete_profile(0);
    expect(names()).toEqual(["Beta", "Gamma"]);
    expect(store.active_profile.char_name).toBe("Gamma");
  });

  it("deleting the active first character moves to the next one", () => {
    store.switch_profile(0);
    store.delete_profile(0);
    expect(store.active_profile.char_name).toBe("Beta");
  });

  it("deleting a character drops its own scan pages only", () => {
    const gamma = store.all_profiles[2];
    store.add_profile({ ...new_char_profile(), roster_id: 1 });
    store.delete_profile(2);
    const config = store.roster_config;
    expect(config.scan_pages.map((p) => p.char_id)).toEqual([null]);
    expect(config.scan_pages.some((p) => p.char_id === gamma.id)).toBe(false);
    expect(config.roster_mats_owned[1]).toBeDefined();
    expect(config.snapshots).toHaveLength(2);
  });

  it("deleting a roster's last character drops all its data", () => {
    store.delete_profile(2);
    const config = store.roster_config;
    for (const record of [
      config.roster_mats_owned,
      config.tradable_mats_owned,
      config.all_regions,
      config.roster_chests,
      config.roster_income,
    ])
      expect(record[1]).toBeUndefined();
    expect(config.scan_pages.every((p) => p.roster_id === 0)).toBe(true);
    expect(config.snapshots.map((s) => s.roster_id)).toEqual([0]);
    expect(config.roster_chests[0]).toHaveLength(1);
  });

  it("a roster added after one was deleted starts empty", () => {
    store.delete_profile(2);
    store.add_roster();
    const config = store.roster_config;
    expect(store.roster_ids).toEqual([0, 1]);
    expect(config.all_regions[1]).toBe("nae");
    expect(config.roster_chests[1]).toBeUndefined();
    expect(config.snapshots.some((s) => s.roster_id === 1)).toBe(false);
    expect(names().at(-1)).toBe("Newchar");
  });

  it("add_char names it uniquely and returns the stored profile", () => {
    const a = store.add_char(1);
    const b = store.add_char(1);
    expect([a.char_name, b.char_name]).toEqual(["Newchar", "Newchar1"]);
    a.char_name = "Delta";
    expect(names().at(-2)).toBe("Delta");
  });

  it("duplicate is an identical character under a new id and name", () => {
    store.duplicate_profile(0);
    const [alpha, , , copy] = store.all_profiles;
    expect(copy.id).not.toBe(alpha.id);
    expect(copy.char_name).toBe("Newchar");
    expect(copy.chests).toEqual(alpha.chests);
    expect(copy.chests).not.toBe(alpha.chests);
    expect(copy.bound_income).toEqual(alpha.bound_income);
    expect(store.active_profile.char_name).toBe("Gamma");
  });

  it("moving keeps the active character", () => {
    store.switch_profile(0);
    store.move_profile(0, 1);
    expect(names()).toEqual(["Beta", "Alpha", "Gamma"]);
    expect(store.active_profile.char_name).toBe("Alpha");
  });
});
