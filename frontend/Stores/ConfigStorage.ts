import LZString from "lz-string";
import {
  ALL_LABELS,
  ALL_WORTHLESS,
  BandPlan,
  bundle_key,
  MATERIALS,
  NUM_BANDS,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import { parse_locale_int } from "@/Utils/InputColumn";
import { debounce } from "@/Utils/Helpers";
import { RosterConfig } from "./RosterConfig";
import { from_saved, SavedColumn, SavedConfig, to_saved } from "./SavedConfig";

export const STORAGE_VERSION = 8;
export const STORAGE_KEY = "HF_CONFIG_V8_COMPRESSED";

// Older keys, newest first. On a version bump, move the current key to the top of
// this list. `version` is only a fallback for pre-V8 saves, which carry no version
// inside the payload.
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
  const keys = [{ key: STORAGE_KEY, version: STORAGE_VERSION }, ...LEGACY_KEYS];
  for (const { key, version } of keys) {
    const data = load_compressed(key);
    if (data === null) continue;
    if (key !== STORAGE_KEY) localStorage.removeItem(key);
    return [data, Number(data.version) || version];
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

// V7 stored one column per tier, with values positional against that tier's label list,
// so a material shared between tiers appeared twice. Prices and owned materials were kept
// in sync by a watcher; a character's bound budget was not, so `winner` says which tier's
// copy survives where the two disagree.
//
// Deliberately ignores the old column's own `keys` array. Nothing before V8 ever read it,
// so it drifted out of order in real saves, while `data` was always interpreted
// positionally against ALL_LABELS. Position is the only trustworthy mapping here.
function v7_merge_columns(
  columns: any,
  winner: number,
  key_of: (label: string) => string = (label) => label,
): SavedColumn {
  const values: Record<string, number> = {};
  const disabled = new Set<string>();
  const tiers = ALL_LABELS.map((_, tier) => tier).filter((t) => t !== winner);
  tiers.push(winner); // the winning tier is applied last, so it overwrites
  for (const tier of tiers) {
    const old = columns?.[tier];
    ALL_LABELS[tier].forEach((label, row) => {
      const parsed = parse_locale_int(String(old?.data?.[row] ?? "0"));
      values[key_of(label)] = Number.isFinite(parsed) ? parsed : 0;
      if (old?.enabled?.[row] === false) {
        disabled.add(key_of(label));
      } else {
        disabled.delete(key_of(label));
      }
    });
  }
  return disabled.size > 0 ? { values, disabled: [...disabled] } : { values };
}

function migrate_7_to_8(data: any): Partial<SavedConfig> {
  // V7 stored one price per material; prices are now per bundle, so each material's
  // single bundle size supplies the key.
  const price_key = (label: string) =>
    bundle_key(label, MATERIALS[label].bundle_sizes[0]);

  const mats_prices: SavedConfig["mats_prices"] = {};
  const selected_bundles: SavedConfig["selected_bundles"] = {};
  for (const region in data.mats_prices ?? {}) {
    const column = v7_merge_columns(data.mats_prices[region], 0, price_key);
    // shard bags were a separate structure with their own prices and chosen size
    const old_shards = data.shard_infos?.[region];
    for (const size in old_shards?.prices ?? {}) {
      column.values[bundle_key("Shards", Number(size))] = parse_locale_int(
        String(old_shards.prices[size]?.data?.[0] ?? "0"),
      );
    }
    mats_prices[region] = column;
    selected_bundles[region] = { Shards: Number(old_shards?.selected) || 3000 };
  }

  const roster_mats_owned: SavedConfig["roster_mats_owned"] = {};
  const tradable_mats_owned: SavedConfig["tradable_mats_owned"] = {};
  for (const roster_id in data.roster_mats_owned ?? {}) {
    roster_mats_owned[roster_id] = v7_merge_columns(
      data.roster_mats_owned[roster_id],
      0,
    );
    tradable_mats_owned[roster_id] = v7_merge_columns(
      data.tradable_mats_owned?.[roster_id],
      0,
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
    const tier = profile.tier === 1 ? 1 : 0;
    // the single V7 special-leap value is in the character's current tier's units
    const special_leaps = parse_locale_int(
      String(profile.special_budget?.data?.[0] ?? "0"),
    );
    return {
      char_name: profile.char_name,
      roster_id: profile.roster_id,
      tier,
      express_event: profile.express_event,
      auto_start_optimizer: profile.auto_start_optimizer,
      lock_fetched_done: profile.lock_fetched_done,
      pretend_30_40_x2_grace: profile.pretend_30_40_x2_grace,
      // V7 encoded the leftover value of each ownership band as a pair of enums that
      // indexed the old breakpoint-merging plan. Two of its four values behaved
      // identically and one was unreachable, so only the two live cases need mapping.
      // 2 was TreatTradableAsBound, the one that credited nothing.
      band_values:
        profile.optimizer_treatment_plan === 2
          ? ALL_WORTHLESS
          : (["Worthless", "Worthless", "TaxedSell"] as BandPlan),
      chance_band: Math.min(
        Math.max(Number(profile.histogram_treatment_plan) || 0, 0),
        NUM_BANDS - 1,
      ),
      normal_grid: profile.normal_grid,
      adv_grid: profile.adv_grid,
      keyed_upgrades,
      special_budget: {
        values: {
          [SPECIAL_LEAP_LABELS[tier]]: Number.isFinite(special_leaps)
            ? special_leaps
            : 0,
        },
      },
      bound_budgets: v7_merge_columns(profile.bound_budgets, tier),
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
    selected_bundles,
    latest_market_data: data.latest_market_data,
    cumulative_graph: data.cumulative_graph,
    show_all_rows: data.show_all_rows,
    auto_deduct_costs: data.auto_deduct_costs,
    auto_fetch: data.auto_fetch,
    enabled_annotations: data.enabled_annotations,
  };
}
