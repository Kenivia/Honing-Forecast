import { validate_input_column_array } from "@/Utils/InputColumn";
import {
  create_default_owned_input_column,
  CURRENT_STORAGE_KEY,
  DEFAULT_ROSTER_CONFIG,
  RosterConfig,
} from "./RosterConfig";
import LZString from "lz-string";
import {
  DEFAULT_CHAR_PROFILE_NO_WORKER,
  init_workers,
  validate_char_profile,
} from "./CharacterProfile";
import { MarketRegions } from "@/Utils/MarketDataFetcher";
import { debounce } from "@/Utils/Helpers";
// just making sure that things are correct, not really necessary i think but oh well
function standard_validation(out: any) {
  out.is_fetching = false;
  for (const key in out.active_mats_prices) {
    validate_input_column_array(
      out.active_mats_prices[key],
      DEFAULT_ROSTER_CONFIG.mats_prices["nae"],
    );
  }
  for (const key in out.roster_mats_owned) {
    validate_input_column_array(
      out.roster_mats_owned[key],
      DEFAULT_ROSTER_CONFIG.roster_mats_owned[0],
    );
    validate_input_column_array(
      out.tradable_mats_owned[key],
      DEFAULT_ROSTER_CONFIG.tradable_mats_owned[0],
    );
  }

  for (let i = 0; i < out.profiles.length; i++) {
    out.profiles[i] = validate_char_profile(out.profiles[i], out, i);
  }
  return out;
}

function migrate_V3(out: any) {
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
  return out;
}

function migrate_V4(out) {
  const v4 = localStorage.getItem("HF_UI_STATE_V4_roster");
  if (v4 !== null) {
    try {
      let parsed = JSON.parse(v4);
      // console.log(parsed);
      out = { ...out, ...parsed };
    } catch (e) {
      console.log("WEEWOO SOMETHING WORNG", e);
    }
    localStorage.removeItem("HF_UI_STATE_V4_roster");
  }
  out.active_profile_index = !out.active_profile_index
    ? 0
    : Math.max(0, Math.min(out.profiles.length - 1, out.active_profile_index)); // just a sanity check, not really necessary

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
  return out;
}

function migrate_V5(out) {
  const v5 = load_compressed("HF_UI_STATE_V5_COMPRESSED");
  if (v5 !== null) {
    out = { ...out, ...v5 };
    localStorage.removeItem("HF_UI_STATE_V5_COMPRESSED");
  }
  delete out["selected_shard_bag_size"]; // 'out' should be DEFAULT_ROSTER_CONFIG and should already have the new shard_infos field
  return out;
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
  let out = (() => {
    try {
      return newest ?? DEFAULT_ROSTER_CONFIG;
    } catch {
      return DEFAULT_ROSTER_CONFIG;
    }
  })();
  // console.log(out);
  out = migrate_V3(out);
  out = migrate_V4(out);
  out = migrate_V5(out);

  out = standard_validation(out);
  const actual_out = { ...DEFAULT_ROSTER_CONFIG, ...out };
  write_roster_config(actual_out);
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
