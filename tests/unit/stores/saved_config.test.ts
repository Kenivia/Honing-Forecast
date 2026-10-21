import { describe, expect, it } from "vitest";
import {
  column_from_saved,
  column_to_saved,
  from_saved,
  to_saved,
} from "@/Stores/SavedConfig";
import { load_roster_config, write_state } from "@/Stores/ConfigStorage";
import { default_roster_config } from "@/Stores/RosterConfig";
import { DEFAULT_CHAR_PROFILE } from "@/Stores/CharacterProfile";
import { input_column_to_num } from "@/Utils/InputColumn";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { NUM_ADV_PIECES } from "@/Utils/Constants";
import { rich_config } from "./fixtures";

const reload = (saved: any) =>
  from_saved(JSON.parse(JSON.stringify({ version: 8, ...saved })));

describe("to_saved / from_saved: V8-only fields", () => {
  it("round trips chests, roster chests, scan pages, snapshots and income", () => {
    const config = rich_config();
    const saved = to_saved(config);
    const back = reload(saved);

    expect(back.profiles.map((p) => p.id)).toEqual(
      config.profiles.map((p) => p.id),
    );
    expect(back.profiles[0].chests).toEqual(config.profiles[0].chests);
    expect(back.profiles[0].income_manual).toBe(true);
    expect(back.profiles[0].bound_income).toEqual({
      Red: 100,
      "Special Leap": 4,
    });
    expect(back.profiles[0].choice_levels).toEqual({ Kazeros: "Hard" });
    expect(back.profiles[2].bound_income).toEqual({ Blue: 9 });
    expect(back.roster_chests).toEqual(config.roster_chests);
    expect(back.scan_pages).toEqual(config.scan_pages);
    expect(back.snapshots).toEqual(config.snapshots);
    expect(back.roster_income).toEqual(config.roster_income);
    expect(back.custom_choices).toEqual(config.custom_choices);
    expect(back.active_profile_index).toBe(2);
    expect(back.all_regions).toEqual({ 0: "nae", 1: "euc" });
    // fixed point from the first load on (the load normalises grids)
    expect(to_saved(reload(to_saved(back)))).toEqual(
      JSON.parse(JSON.stringify(to_saved(back))),
    );
  });

  it("load, save, load again through localStorage keeps every field", () => {
    write_state({ roster_config: rich_config() });
    const first = load_roster_config();
    write_state({ roster_config: first });
    const second = load_roster_config();
    expect(to_saved(second)).toEqual(to_saved(first));
    expect(second.scan_pages).toHaveLength(2);
    expect(
      second.snapshots[1].chars[first.profiles[2].id].results.abc.gold,
    ).toBe(123);
    expect(input_column_to_num(second.roster_mats_owned[0]).Red).toBe(1234);
    expect(input_column_to_num(second.tradable_mats_owned[1]).Blue).toBe(77);
  });

  it("a save without the V8-only fields loads them empty or off", () => {
    const saved: any = to_saved(rich_config());
    for (const key of [
      "roster_chests",
      "scan_pages",
      "snapshots",
      "roster_income",
      "custom_choices",
    ])
      delete saved[key];
    for (const p of saved.profiles)
      for (const key of [
        "chests",
        "income_manual",
        "bound_income",
        "choice_levels",
      ])
        delete p[key];
    const back = reload(saved);
    expect(back.roster_chests).toEqual({});
    expect(back.scan_pages).toEqual([]);
    expect(back.snapshots).toEqual([]);
    expect(back.roster_income).toEqual({});
    expect(back.custom_choices).toEqual({});
    expect(back.profiles[0].chests).toEqual([]);
    expect(back.profiles[0].income_manual).toBe(false);
    expect(back.profiles[0].bound_income).toEqual({});
    expect(back.profiles[0].choice_levels).toEqual({});
  });

  it("non-array scan pages, snapshots and chests load empty", () => {
    const saved: any = to_saved(rich_config());
    saved.scan_pages = { a: 1 };
    saved.snapshots = "x";
    saved.profiles[0].chests = {};
    const back = reload(saved);
    expect(back.scan_pages).toEqual([]);
    expect(back.snapshots).toEqual([]);
    expect(back.profiles[0].chests).toEqual([]);
  });

  it("an empty save gives the default config", () => {
    const back = from_saved({});
    const fresh = default_roster_config();
    expect(back.profiles).toHaveLength(1);
    expect(back.profiles[0].id).toMatch(/[0-9a-f-]{36}/);
    expect(to_saved({ ...back, profiles: [] })).toEqual(
      to_saved({ ...fresh, profiles: [] }),
    );
  });
});

describe("from_saved: repairs", () => {
  it("a profile on a roster with no materials moves to the first roster", () => {
    const saved: any = to_saved(rich_config());
    saved.profiles[2].roster_id = 9;
    expect(reload(saved).profiles[2].roster_id).toBe(0);
  });

  it("duplicate names get a suffix, ids stay", () => {
    const saved: any = to_saved(rich_config());
    saved.profiles[1].char_name = "Alpha";
    const back = reload(saved);
    expect(back.profiles.map((p) => p.char_name)).toEqual([
      "Alpha",
      "Alpha1",
      "Gamma",
    ]);
    expect(back.profiles[1].id).toBe(saved.profiles[1].id);
  });

  it("active index is clamped to the profile list", () => {
    const saved: any = to_saved(rich_config());
    saved.active_profile_index = 10;
    expect(reload(saved).active_profile_index).toBe(2);
    saved.active_profile_index = -3;
    expect(reload(saved).active_profile_index).toBe(0);
  });

  it("band values: invalid falls back, decreasing is clamped up", () => {
    const saved: any = to_saved(rich_config());
    saved.profiles[0].band_values = ["Worthless", "Bogus", "Market"];
    saved.profiles[1].band_values = ["Market", "Worthless", "TaxedSell"];
    saved.profiles[2].band_values = ["Worthless", "Market"];
    const back = reload(saved);
    expect(back.profiles[0].band_values).toEqual(
      DEFAULT_CHAR_PROFILE.band_values,
    );
    expect(back.profiles[1].band_values).toEqual([
      "Market",
      "Market",
      "Market",
    ]);
    expect(back.profiles[2].band_values).toEqual(
      DEFAULT_CHAR_PROFILE.band_values,
    );
  });

  it("chance band and tier are clamped", () => {
    const saved: any = to_saved(rich_config());
    saved.profiles[0].chance_band = 7;
    saved.profiles[1].tier = 5;
    const back = reload(saved);
    expect(back.profiles[0].chance_band).toBe(2);
    expect(back.profiles[1].tier).toBe(0);
  });

  it("levels below the tier floor load as done, except the vambrace", () => {
    const saved: any = to_saved(rich_config());
    for (const p of saved.profiles)
      p.normal_grid = p.normal_grid.map((row) =>
        row.map(() => UpgradeStatus.Want),
      );
    const back = reload(saved);
    const t0 = back.profiles[0].normal_grid; // floor 10
    const t1 = back.profiles[1].normal_grid; // floor 11
    expect(t0[0][9]).toBe(UpgradeStatus.FetchedDone);
    expect(t0[0][10]).toBe(UpgradeStatus.Want);
    expect(t1[0][10]).toBe(UpgradeStatus.FetchedDone);
    expect(t1[0][11]).toBe(UpgradeStatus.Want);
    expect(t0[NUM_ADV_PIECES][0]).toBe(UpgradeStatus.Want);
  });

  it("an old save's enabled_annotations is ignored", () => {
    const saved: any = to_saved(rich_config());
    saved.enabled_annotations = [false, false, true, true];
    expect(reload(saved)).not.toHaveProperty("enabled_annotations");
  });

  it("selected bundle sizes that do not exist fall back to the default", () => {
    const saved: any = to_saved(rich_config());
    saved.selected_bundles.nae.Shards = 1234;
    saved.selected_bundles.euc.Shards = 1000;
    const back = reload(saved);
    expect(back.selected_bundles.nae.Shards).toBe(3000);
    expect(back.selected_bundles.euc.Shards).toBe(1000);
  });

  it("a roster missing its region gets nae", () => {
    const saved: any = to_saved(rich_config());
    delete saved.all_regions[1];
    expect(reload(saved).all_regions[1]).toBe("nae");
  });
});

describe("columns", () => {
  it("disabled labels round trip, and disabling one disables its linked label", () => {
    const column = structuredClone(DEFAULT_CHAR_PROFILE.bound_budgets);
    column.enabled["Red"] = false;
    const saved = column_to_saved(column);
    expect(saved.disabled).toContain("Red");
    expect(saved.disabled).toContain("Shards");
    const back = column_from_saved(saved, DEFAULT_CHAR_PROFILE.bound_budgets);
    expect(back.enabled["Red"]).toBe(false);
    expect(back.enabled["Serca Red"]).toBe(false);
    expect(back.enabled["Blue"]).toBe(true);
  });

  it("values are clamped to the template's upper bound", () => {
    const back = column_from_saved(
      { values: { "Special Leap": 99999999 } },
      DEFAULT_CHAR_PROFILE.special_budget,
    );
    expect(input_column_to_num(back)["Special Leap"]).toBe(33333);
  });

  it("an unknown label in the save is dropped", () => {
    const back = column_from_saved(
      { values: { Nonsense: 5, Red: 3 } },
      DEFAULT_CHAR_PROFILE.bound_budgets,
    );
    expect("Nonsense" in back.values).toBe(false);
    expect(input_column_to_num(back).Red).toBe(3);
  });

  it("a disabled cell keeps its value in the save", () => {
    const column = structuredClone(DEFAULT_CHAR_PROFILE.bound_budgets);
    column.values.Red = "42";
    column.enabled.Red = false;
    expect(column_to_saved(column).values.Red).toBe(42);
  });
});
