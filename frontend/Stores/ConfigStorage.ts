import { InputColumn, validate_input_column_array } from "@/Utils/InputColumn";
import {
  create_default_owned_input_column,
  DEFAULT_ROSTER_CONFIG,
  RosterConfig,
} from "./RosterConfig";
import LZString from "lz-string";
import {
  CharProfile,
  DEFAULT_CHAR_PROFILE_NO_WORKER,
  init_workers,
  validate_char_profile,
} from "./CharacterProfile";
import { MarketRegions } from "@/Utils/MarketDataFetcher";
import { debounce } from "@/Utils/Helpers";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { NUM_PIECES } from "@/Utils/Constants";

export const CURRENT_STORAGE_KEY = "HF_CONFIG_V7_COMPRESSED";
export const CURRENT_VERSION_NUMBER = 7;

function standard_validation(out: any) {
  out.is_fetching = false;
  for (const key in out.mats_prices) {
    out.mats_prices[key] = validate_input_column_array(
      out.mats_prices[key],
      DEFAULT_ROSTER_CONFIG.mats_prices["nae"],
    );
  }
  for (const key in out.roster_mats_owned) {
    out.roster_mats_owned[key] = validate_input_column_array(
      out.roster_mats_owned[key],
      DEFAULT_ROSTER_CONFIG.roster_mats_owned[0],
    );
    out.tradable_mats_owned[key] = validate_input_column_array(
      out.tradable_mats_owned[key],
      DEFAULT_ROSTER_CONFIG.tradable_mats_owned[0],
    );
  }
  // console.log(structuredClone(out.mats_prices));
  for (let i = 0; i < out.profiles.length; i++) {
    out.profiles[i] = validate_char_profile(out.profiles[i], out, i);
  }

  return out;
}

function migrate_V3(out: any, _: number): [any, number] {
  const old_char_profiles = localStorage.getItem(
    "HF_UI_STATE_V3_char_profiles",
  );
  if (old_char_profiles !== null) {
    try {
      let parsed = JSON.parse(old_char_profiles);
      out.profiles = parsed.profiles;
    } catch {
      out.profiles = [init_workers(DEFAULT_CHAR_PROFILE_NO_WORKER)];
    }
    localStorage.removeItem("HF_UI_STATE_V3_char_profiles");
  }

  const old_roster = localStorage.getItem("HF_UI_STATE_V3_roster");
  if (old_roster !== null) {
    try {
      let parsed = JSON.parse(old_roster);
      out.roster_mats_owned = { 0: parsed.roster_mats_owned };
      out.tradable_mats_owned = { 0: parsed.tradable_mats_owned };
    } catch {
      out.roster_mats_owned = { 0: create_default_owned_input_column() };
      out.tradable_mats_owned = { 0: create_default_owned_input_column() };
    }
    localStorage.removeItem("HF_UI_STATE_V3_roster");
  }

  return [out, 4];
}

function migrate_V4(out, version: number): [any, number] {
  const v4 = localStorage.getItem("HF_UI_STATE_V4_roster");
  if (v4 !== null) {
    version = 4;
    try {
      let parsed = JSON.parse(v4);
      // console.log(parsed);
      out = { ...out, ...parsed };
    } catch (e) {
      console.log("WEEWOO SOMETHING WORNG", e);
    }
    localStorage.removeItem("HF_UI_STATE_V4_roster");
  }
  if (version == 4) {
    out.active_profile_index = !out.active_profile_index
      ? 0
      : Math.max(
          0,
          Math.min(out.profiles.length - 1, out.active_profile_index),
        ); // just a sanity check, not really necessary

    if (out.region !== undefined) {
      const region: MarketRegions = out.region.toLowerCase();
      out.all_regions = DEFAULT_ROSTER_CONFIG.all_regions;
      for (let index = 0; index < out.profiles.length; index++) {
        out.all_regions[out.profiles[index].roster_id] = region;
      }
      out.mats_prices = DEFAULT_ROSTER_CONFIG.mats_prices;
      delete out["region"];
      // out.selected_shard_bag_size = DEFAULT_ROSTER_CONFIG.selected_shard_bag_size;
    }
  }

  return [out, version + 1];
}

function migrate_V5(out, version: number): [any, number] {
  const v5 = load_compressed("HF_UI_STATE_V5_COMPRESSED");
  if (v5 !== null) {
    version = 5;
    out = { ...out, ...v5 };
    localStorage.removeItem("HF_UI_STATE_V5_COMPRESSED");
  }
  if (version == 5) {
    delete out["selected_shard_bag_size"]; // 'out' should be DEFAULT_ROSTER_CONFIG and should already have the new shard_infos field
  }
  return [out, version + 1];
}
function migrate_V6(out, version: number): [any, number] {
  const v6 = load_compressed("HF_CONFIG_V6_COMPRESSED");
  // console.log(v6);
  if (v6 !== null) {
    version = 6;
    out = { ...out, ...v6 };
    localStorage.removeItem("HF_CONFIG_V6_COMPRESSED");
  }
  if (version == 6) {
    let swap_key_map = [
      [
        0, 1, 2, 3, 4, 5, 6, 15, 7, 16, 8, 17, 9, 18, 10, 19, 11, 20, 12, 21,
        13, 22, 14,
      ],
      [0, 1, 2, 3, 4, 5, 6, 8, 7],
    ];
    function swap_keys(input_column_array: InputColumn[]): InputColumn[] {
      for (const [tier, input_column] of input_column_array.entries()) {
        const copy = structuredClone(input_column);
        for (const [row, dest] of swap_key_map[tier].entries()) {
          input_column.data[row] = copy.data[dest];
          input_column.keys[row] = copy.keys[dest];
          input_column.upper_bound[row] = copy.upper_bound[dest];
          input_column.enabled[row] = copy.enabled[dest];
        }
      }
      return input_column_array;
    }

    for (const key in out.mats_prices) {
      out.mats_prices[key] = swap_keys(out.mats_prices[key]);
    }

    for (const key in out.roster_mats_owned) {
      out.roster_mats_owned[key] = swap_keys(out.roster_mats_owned[key]);
      out.tradable_mats_owned[key] = swap_keys(out.tradable_mats_owned[key]);
    }
    for (const profile of out.profiles) {
      if (profile.normal_grid.length < NUM_PIECES) {
        profile.normal_grid.push(
          Array.from({ length: 25 }).fill(UpgradeStatus.NotYet),
        );
      }
      // if (profile.char_name == "Toneema") {
      //   console.log(structuredClone(profile.bound_budgets));
      // }

      profile.bound_budgets = swap_keys(profile.bound_budgets);
      // if (profile.char_name == "Toneema") {
      //   // console.log(structuredClone(profile.bound_budgets));
      // }
      profile.leftover_price = swap_keys(profile.leftover_price);
    }
    // console.log(structuredClone(out.profiles));
  }
  return [out, version + 1];
}

function load_compressed(key: string): any {
  const compressed = localStorage.getItem(key);
  return compressed !== null
    ? JSON.parse(LZString.decompressFromUTF16(compressed))
    : null;
}
export function load_roster_config(): RosterConfig {
  // console.log(newest_version);
  const newest = load_compressed(CURRENT_STORAGE_KEY);
  let out = newest ? newest : DEFAULT_ROSTER_CONFIG;

  let version = CURRENT_VERSION_NUMBER;
  [out, version] = migrate_V3(out, version);
  [out, version] = migrate_V4(out, version);
  [out, version] = migrate_V5(out, version);
  [out] = migrate_V6(out, version);

  out = standard_validation(out);
  const actual_out = { ...DEFAULT_ROSTER_CONFIG, ...out };
  write_roster_config(actual_out);
  // console.log(actual_out);
  return actual_out;
}

function write_roster_config(roster_config: RosterConfig) {
  const json = stringifyOmit(roster_config, [
    "optimizer_worker_bundle",
    "histogram_worker_bundle",
    "optimizer_override",
    "budget_snapshot",
    "is_slider_update",
    "adv_cache",
  ]);
  localStorage.setItem(CURRENT_STORAGE_KEY, LZString.compressToUTF16(json));
}
export function write_state(state) {
  // console.log("writing");
  try {
    write_roster_config(state.roster_config);
  } catch {
    console.log(JSON.stringify(state.roster_config));
  }
}
function stringifyOmit(obj: RosterConfig, keys: string[]): string {
  const omit = new Set(keys);
  return JSON.stringify(obj, (key, value) =>
    omit.has(key) ? undefined : value,
  );
}

export const debounced_write_roster_config = debounce(write_state, 500);
