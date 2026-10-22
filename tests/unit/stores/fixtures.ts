import type { ManifestChest } from "@/Utils/Chest";
import type { ScanPage, Snapshot } from "@/Utils/Scans";
import { default_roster_config, RosterConfig } from "@/Stores/RosterConfig";
import { new_char_profile } from "@/Stores/CharacterProfile";
import { create_default_owned_input_column } from "@/Stores/RosterConfig";

export function chest(title: string, band = 0, count = 1): ManifestChest {
  return {
    key: title,
    title,
    band,
    count,
    options: [{ Red: 100 }, { Blue: 200, Gold: 5 }],
    icons: ["icon_a"],
    obtain_all: false,
  };
}

export function scan_page(roster_id: string, char_id: string | null): ScanPage {
  return {
    roster_id,
    char_id,
    window: "Roster",
    page: 2,
    at: 1700000000000,
    by: "someid",
    materials: { Red: [1, 2, 3] },
    chests: [chest("Page chest")],
    random: [],
    unknown: [],
  } as ScanPage;
}

export function snapshot(roster_id: string, char_id: string): Snapshot {
  return {
    at: 1700000001000,
    kind: "Manual",
    roster_id,
    roster_owned: { Red: 10 },
    tradable: { Blue: 20 },
    roster_chests: [chest("Snap roster chest", 1)],
    chars: {
      [char_id]: {
        name: "Alpha",
        bound: { Red: 5 },
        special: { "Special Leap": 3 },
        chests: [chest("Snap char chest")],
        results: { abc: { gains: { Red: [1, 2, 3] }, gold: 123, leftover: 4 } },
      },
    },
    prices: { "Red 1": 7 },
    bundles: { Shards: 2000 },
    ignored: true,
    breaks: false,
  };
}

// Two rosters ("0" and "1"), three characters, every V8-only field filled in.
export function rich_config(): RosterConfig {
  const config = default_roster_config();
  config.roster_mats_owned = {
    0: create_default_owned_input_column(),
    1: create_default_owned_input_column(),
  };
  config.tradable_mats_owned = {
    0: create_default_owned_input_column(),
    1: create_default_owned_input_column(),
  };
  config.all_regions = { 0: "nae", 1: "euc" };
  const a = { ...new_char_profile(), char_name: "Alpha", roster_id: "0" };
  const b = { ...new_char_profile(), char_name: "Beta", roster_id: "0", tier: 1 };
  const c = { ...new_char_profile(), char_name: "Gamma", roster_id: "1" };
  a.chests = [chest("Alpha chest"), chest("Alpha chest 2", 0, 3)];
  a.income_manual = true;
  a.bound_income = { Red: 100, "Special Leap": 4 };
  a.choice_levels = { Kazeros: "Hard" };
  c.bound_income = { Blue: 9 };
  config.profiles = [a, b, c];
  config.active_profile_index = 2;
  config.roster_chests = { 0: [chest("Roster chest", 1, 2)], 1: [] };
  config.scan_pages = [scan_page("0", null), scan_page("1", c.id)];
  config.snapshots = [snapshot("0", a.id), snapshot("1", c.id)];
  config.custom_choices = {
    0: { band: 1, names: ["A", "B"], options: [{ Red: 5, Gold: -10 }, {}] },
  };
  config.roster_income = {
    0: [{ Red: 1000 }, { Blue: 50 }],
    1: [{}, { Leaps: 2 }],
  };
  config.roster_mats_owned[0].values.Red = "1,234";
  config.tradable_mats_owned[1].values.Blue = "77";
  return config;
}
