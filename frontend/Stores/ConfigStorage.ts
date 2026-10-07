import LZString from "lz-string";
import { ALL_LABELS } from "@/Utils/Constants";
import { parse_locale_int } from "@/Utils/InputColumn";
import { debounce } from "@/Utils/Helpers";
import { RosterConfig } from "./RosterConfig";
import { from_saved, SavedColumn, SavedConfig, to_saved } from "./SavedConfig";

export const STORAGE_KEY = "HF_CONFIG";
export const STORAGE_VERSION = 8;

// Saves written before the version number moved inside the payload, newest first.
const LEGACY_KEYS: { key: string; version: number }[] = [
  { key: "HF_CONFIG_V7_COMPRESSED", version: 7 },
  { key: "HF_CONFIG_V6_COMPRESSED", version: 6 },
  { key: "HF_UI_STATE_V5_COMPRESSED", version: 5 },
];

// V3 and V4 saves are no longer migrated. Removed so they stop taking up quota.
const ABANDONED_KEYS = [
  "HF_UI_STATE_V3_char_profiles",
  "HF_UI_STATE_V3_roster",
  "HF_UI_STATE_V4_roster",
];

const MIGRATIONS: Record<number, (data: any) => any> = {
  5: migrate_5_to_6,
  6: migrate_6_to_7,
  7: migrate_7_to_8,
};

// ============================================================================
// Load
// ============================================================================

function load_compressed(key: string): any {
  const compressed = localStorage.getItem(key);
  if (compressed === null) {
    return null;
  }
  try {
    return JSON.parse(LZString.decompressFromUTF16(compressed));
  } catch {
    return null;
  }
}

function read_newest_save(): [any, number] | null {
  const current = load_compressed(STORAGE_KEY);
  if (current !== null) {
    return [current, Number(current.version) || STORAGE_VERSION];
  }
  for (const { key, version } of LEGACY_KEYS) {
    const data = load_compressed(key);
    if (data !== null) {
      localStorage.removeItem(key);
      return [data, version];
    }
  }
  return null;
}

function migrate_to_current(data: any, version: number): any {
  let out = data;
  let v = version;
  while (v < STORAGE_VERSION) {
    out = MIGRATIONS[v](out);
    v += 1;
  }
  return out;
}

export function load_roster_config(): RosterConfig {
  for (const key of ABANDONED_KEYS) {
    localStorage.removeItem(key);
  }
  const found = read_newest_save();
  if (found === null) {
    return from_saved({});
  }
  const [data, version] = found;
  try {
    return from_saved(migrate_to_current(data, version));
  } catch (e) {
    console.error("could not load saved config, starting fresh", e);
    return from_saved({});
  }
}

// ============================================================================
// Save
// ============================================================================

function serialize(config: RosterConfig): string {
  return JSON.stringify({ version: STORAGE_VERSION, ...to_saved(config) });
}

export function write_state(state: { roster_config: RosterConfig }) {
  try {
    localStorage.setItem(
      STORAGE_KEY,
      LZString.compressToUTF16(serialize(state.roster_config)),
    );
  } catch (e) {
    console.error("could not save config", e);
  }
}

export const debounced_write_roster_config = debounce(write_state, 500);

// ============================================================================
// Export / import
// ============================================================================

export function export_config(config: RosterConfig): string {
  return JSON.stringify(
    { version: STORAGE_VERSION, ...to_saved(config) },
    null,
    2,
  );
}

// Runs the same migration chain as a load, so older backups still import.
export function import_config(text: string): RosterConfig {
  const data = JSON.parse(text);
  const version = Number(data.version) || STORAGE_VERSION;
  if (version > STORAGE_VERSION) {
    throw new Error(
      `backup is from a newer version (${version}) than this site understands (${STORAGE_VERSION})`,
    );
  }
  return from_saved(migrate_to_current(data, version));
}

// ============================================================================
// Migrations
//
// 5 -> 6 and 6 -> 7 operate on the old shape, where the whole store was serialised
// as-is and material values were positional. 7 -> 8 converts to the label-keyed DTO,
// after which reordering material rows stops being a migration at all.
// ============================================================================

function migrate_5_to_6(data: any): any {
  delete data.selected_shard_bag_size; // replaced by shard_infos
  return data;
}

function migrate_6_to_7(data: any): any {
  // Material rows were reordered: juices went from grouped-by-slot to interleaved.
  const swap_key_map = [
    [
      0, 1, 2, 3, 4, 5, 6, 15, 7, 16, 8, 17, 9, 18, 10, 19, 11, 20, 12, 21, 13,
      22, 14,
    ],
    [0, 1, 2, 3, 4, 5, 6, 8, 7],
  ];

  function swap(columns: any[]): any[] {
    if (!Array.isArray(columns)) {
      return columns;
    }
    for (const [tier, column] of columns.entries()) {
      const map = swap_key_map[tier];
      if (!map || !column) {
        continue;
      }
      const copy = structuredClone(column);
      for (const [row, source] of map.entries()) {
        for (const field of ["data", "keys", "upper_bound", "enabled"]) {
          if (Array.isArray(column[field])) {
            column[field][row] = copy[field][source];
          }
        }
      }
    }
    return columns;
  }

  for (const key in data.mats_prices ?? {}) {
    data.mats_prices[key] = swap(data.mats_prices[key]);
  }
  for (const key in data.roster_mats_owned ?? {}) {
    data.roster_mats_owned[key] = swap(data.roster_mats_owned[key]);
    data.tradable_mats_owned[key] = swap(data.tradable_mats_owned[key]);
  }
  for (const profile of data.profiles ?? []) {
    // the vambrace row did not exist before V7
    if (Array.isArray(profile.normal_grid) && profile.normal_grid.length < 7) {
      profile.normal_grid.push(Array(25).fill(2 /* NotYet */));
    }
    profile.bound_budgets = swap(profile.bound_budgets);
    profile.leftover_price = swap(profile.leftover_price);
  }
  return data;
}

// Deliberately ignores the old column's own `keys` array. Nothing before V8 ever read
// it, so it drifted out of order in real saves, while `data` was always interpreted
// positionally against ALL_LABELS. Position is the only trustworthy mapping here.
function v7_column(old: any, keys: string[]): SavedColumn {
  const values: Record<string, number> = {};
  const disabled: string[] = [];
  keys.forEach((key, row) => {
    const parsed = parse_locale_int(String(old?.data?.[row] ?? "0"));
    values[key] = Number.isFinite(parsed) ? parsed : 0;
    if (old?.enabled?.[row] === false) {
      disabled.push(key);
    }
  });
  return disabled.length > 0 ? { values, disabled } : { values };
}

const v7_columns = (old: any): SavedColumn[] =>
  ALL_LABELS.map((labels, tier) => v7_column(old?.[tier], labels));

function migrate_7_to_8(data: any): Partial<SavedConfig> {
  const mats_prices: SavedConfig["mats_prices"] = {};
  for (const region in data.mats_prices ?? {}) {
    mats_prices[region] = v7_columns(data.mats_prices[region]);
  }

  const shard_infos: SavedConfig["shard_infos"] = {};
  for (const region in data.shard_infos ?? {}) {
    const old = data.shard_infos[region];
    const prices: Record<string, number> = {};
    for (const size in old?.prices ?? {}) {
      prices[size] = parse_locale_int(
        String(old.prices[size]?.data?.[0] ?? "0"),
      );
    }
    shard_infos[region] = { selected: old?.selected ?? 3000, prices };
  }

  const roster_mats_owned: SavedConfig["roster_mats_owned"] = {};
  const tradable_mats_owned: SavedConfig["tradable_mats_owned"] = {};
  for (const roster_id in data.roster_mats_owned ?? {}) {
    roster_mats_owned[roster_id] = v7_columns(
      data.roster_mats_owned[roster_id],
    );
    tradable_mats_owned[roster_id] = v7_columns(
      data.tradable_mats_owned?.[roster_id],
    );
  }

  const profiles = (data.profiles ?? []).map((profile: any) => {
    const keyed_upgrades: Record<string, any> = {};
    for (const [key, upgrade] of Object.entries(profile.keyed_upgrades ?? {})) {
      if (upgrade !== null && typeof upgrade === "object") {
        const { used_materials: _drop, ...rest } = upgrade as any;
        keyed_upgrades[key] = rest;
      }
    }
    return {
      char_name: profile.char_name,
      roster_id: profile.roster_id,
      tier: profile.tier,
      express_event: profile.express_event,
      auto_start_optimizer: profile.auto_start_optimizer,
      lock_fetched_done: profile.lock_fetched_done,
      pretend_30_40_x2_grace: profile.pretend_30_40_x2_grace,
      optimizer_treatment_plan: profile.optimizer_treatment_plan,
      histogram_treatment_plan: profile.histogram_treatment_plan,
      normal_grid: profile.normal_grid,
      adv_grid: profile.adv_grid,
      keyed_upgrades,
      special_budget: v7_column(profile.special_budget, ["Special Leap"]),
      bound_budgets: v7_columns(profile.bound_budgets),
      leftover_price: v7_columns(profile.leftover_price),
    };
  });

  return {
    version: STORAGE_VERSION,
    profiles,
    active_profile_index: data.active_profile_index,
    last_seen_version: data.last_seen_version,
    mats_prices,
    roster_mats_owned,
    tradable_mats_owned,
    all_regions: data.all_regions,
    shard_infos,
    latest_market_data: data.latest_market_data,
    cumulative_graph: data.cumulative_graph,
    show_all_rows: data.show_all_rows,
    auto_deduct_costs: data.auto_deduct_costs,
    auto_fetch: data.auto_fetch,
    enabled_annotations: data.enabled_annotations,
  };
}
