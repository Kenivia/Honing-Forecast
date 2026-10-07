import { NUM_ADV_PIECES } from "@/Utils/Constants";
import {
  create_input_column,
  InputColumn,
  parse_input,
} from "@/Utils/InputColumn";
import {
  CharProfile,
  DEFAULT_CHAR_PROFILE,
  TreatmentPlan,
} from "./CharacterProfile";
import { default_roster_config, RosterConfig } from "./RosterConfig";
import {
  DEFAULT_SHARD_INFO,
  MarketRegions,
  ShardInfo,
} from "@/Utils/MarketDataFetcher";
import {
  grids_to_keyed,
  KeyedUpgrades,
  OneUpgradeInput,
  StatusGrid,
  UpgradeStatus,
} from "@/Utils/KeyedUpgrades";
import { get_valid_status_grid } from "@/Utils/StatusGrid";
import { format_char_name } from "@/Utils/Helpers";

// ============================================================================
// The persisted shape.
//
// Two rules, and the whole module follows from them:
//  1. Nothing derivable from Constants.ts is stored. Keys, bounds, column type and
//     row order all come from the constants at load time.
//  2. Material values are keyed by label, never by row index. Reordering, inserting
//     or removing a row in Constants.ts is therefore not a save-shape change.
// ============================================================================

export interface SavedColumn {
  values: Record<string, number>;
  disabled?: string[]; // complete list of disabled labels; absent means all enabled
}

// used_materials is left out: it is indexed by material row, and it is meaningless
// without budget_snapshot, which is session-only.
export type SavedUpgrade = Omit<OneUpgradeInput, "used_materials">;

export interface SavedProfile {
  char_name: string;
  roster_id: number;
  tier: number;
  express_event: boolean;
  auto_start_optimizer: boolean;
  lock_fetched_done: boolean;
  pretend_30_40_x2_grace: boolean;
  optimizer_treatment_plan: TreatmentPlan;
  histogram_treatment_plan: TreatmentPlan;
  normal_grid: StatusGrid;
  adv_grid: StatusGrid;
  keyed_upgrades: Record<string, SavedUpgrade>;
  special_budget: SavedColumn;
  bound_budgets: SavedColumn[]; // by tier
  leftover_price: SavedColumn[]; // by tier
}

export interface SavedShardInfo {
  selected: number;
  prices: Record<string, number>; // bag size -> price
}

export interface SavedConfig {
  version: number;
  profiles: SavedProfile[];
  active_profile_index: number;
  last_seen_version: string;
  mats_prices: Partial<Record<MarketRegions, SavedColumn[]>>;
  roster_mats_owned: Record<string, SavedColumn[]>;
  tradable_mats_owned: Record<string, SavedColumn[]>;
  all_regions: Record<string, MarketRegions>;
  shard_infos: Partial<Record<MarketRegions, SavedShardInfo>>;
  latest_market_data: Partial<Record<MarketRegions, [number, any]>>;
  cumulative_graph: boolean;
  show_all_rows: boolean;
  auto_deduct_costs: boolean;
  auto_fetch: boolean;
  enabled_annotations: boolean[];
}

// ============================================================================
// Columns
// ============================================================================

export function column_to_saved(column: InputColumn): SavedColumn {
  const values: Record<string, number> = {};
  column.keys.forEach((key, row) => {
    values[key] = parse_input(column, row, column.data[row], true);
  });
  const disabled = column.keys.filter((_, row) => !column.enabled[row]);
  return disabled.length > 0 ? { values, disabled } : { values };
}

// `template` is the default column built from Constants, and supplies everything the
// save does not carry. A label missing from the save falls back to the template's
// value, so a newly added material gets its fallback price rather than zero.
export function column_from_saved(
  saved: SavedColumn | undefined,
  template: InputColumn,
): InputColumn {
  const out = create_input_column(
    template.type,
    template.keys.slice(),
    template.data.slice(),
    template.upper_bound.slice(),
    template.enabled.slice(),
  );
  if (!saved) {
    return out;
  }
  const disabled = new Set(saved.disabled ?? []);
  out.enabled = template.keys.map((key) => !disabled.has(key));
  out.data = template.keys.map((key, row) =>
    saved.values?.[key] === undefined
      ? template.data[row]
      : parse_input(out, row, String(saved.values[key]), true).toLocaleString(),
  );
  return out;
}

const columns_to_saved = (columns: InputColumn[]): SavedColumn[] =>
  columns.map(column_to_saved);

const columns_from_saved = (
  saved: SavedColumn[] | undefined,
  templates: InputColumn[],
): InputColumn[] =>
  templates.map((template, tier) => column_from_saved(saved?.[tier], template));

// ============================================================================
// Profiles
// ============================================================================

function profile_to_saved(profile: CharProfile): SavedProfile {
  const keyed_upgrades: Record<string, SavedUpgrade> = {};
  for (const [key, upgrade] of Object.entries(profile.keyed_upgrades)) {
    const { used_materials: _drop, ...rest } = upgrade;
    keyed_upgrades[key] = rest;
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
    special_budget: column_to_saved(profile.special_budget),
    bound_budgets: columns_to_saved(profile.bound_budgets),
    leftover_price: columns_to_saved(profile.leftover_price),
  };
}

function profile_from_saved(
  saved: Partial<SavedProfile>,
  index: number,
  earlier: CharProfile[],
  roster_ids: number[],
): CharProfile {
  const defaults = structuredClone(DEFAULT_CHAR_PROFILE);
  const tier =
    saved.tier === 0 || saved.tier === 1 ? saved.tier : defaults.tier;

  const roster_id =
    saved.roster_id !== undefined &&
    roster_ids.includes(Number(saved.roster_id))
      ? Number(saved.roster_id)
      : (roster_ids[0] ?? 0);

  // Levels below the tier's floor are always already done, except on the vambrace.
  const floor = tier === 0 ? 10 : 11;
  const normal_grid = get_valid_status_grid(
    saved.normal_grid,
    defaults.normal_grid,
  ).map((row, piece) =>
    piece === NUM_ADV_PIECES
      ? row
      : row.map((cell, col) =>
          col < floor ? UpgradeStatus.FetchedDone : cell,
        ),
  );
  const adv_grid = get_valid_status_grid(saved.adv_grid, defaults.adv_grid);

  const keyed_upgrades: KeyedUpgrades = {};
  for (const [key, upgrade] of Object.entries(saved.keyed_upgrades ?? {})) {
    keyed_upgrades[key] = { ...upgrade, used_materials: null };
  }

  return {
    ...defaults,
    char_name: format_char_name(
      saved.char_name ?? defaults.char_name,
      index,
      earlier,
    ),
    roster_id,
    tier,
    express_event: saved.express_event ?? defaults.express_event,
    auto_start_optimizer:
      saved.auto_start_optimizer ?? defaults.auto_start_optimizer,
    lock_fetched_done: saved.lock_fetched_done ?? defaults.lock_fetched_done,
    pretend_30_40_x2_grace:
      saved.pretend_30_40_x2_grace ?? defaults.pretend_30_40_x2_grace,
    optimizer_treatment_plan:
      saved.optimizer_treatment_plan ?? defaults.optimizer_treatment_plan,
    histogram_treatment_plan:
      saved.histogram_treatment_plan ?? defaults.histogram_treatment_plan,
    normal_grid,
    adv_grid,
    // rebuilt from the grids so the two can never disagree
    keyed_upgrades: grids_to_keyed(normal_grid, adv_grid, keyed_upgrades, tier),
    special_budget: column_from_saved(
      saved.special_budget,
      defaults.special_budget,
    ),
    bound_budgets: columns_from_saved(
      saved.bound_budgets,
      defaults.bound_budgets,
    ),
    leftover_price: columns_from_saved(
      saved.leftover_price,
      defaults.leftover_price,
    ),
  };
}

// ============================================================================
// Shards
// ============================================================================

function shard_info_to_saved(info: ShardInfo): SavedShardInfo {
  const prices: Record<string, number> = {};
  for (const [size, column] of Object.entries(info.prices)) {
    prices[size] = parse_input(column, 0, column.data[0], true);
  }
  return { selected: info.selected, prices };
}

function shard_info_from_saved(saved: SavedShardInfo | undefined): ShardInfo {
  const out = structuredClone(DEFAULT_SHARD_INFO);
  if (!saved) {
    return out;
  }
  if (Object.hasOwn(out.prices, saved.selected)) {
    out.selected = saved.selected;
  }
  for (const size of Object.keys(out.prices)) {
    const value = saved.prices?.[size];
    if (value !== undefined) {
      out.prices[size].data[0] = Number(value).toLocaleString();
    }
  }
  return out;
}

// ============================================================================
// Whole config
// ============================================================================

export function to_saved(config: RosterConfig): Omit<SavedConfig, "version"> {
  const mats_prices: SavedConfig["mats_prices"] = {};
  for (const [region, columns] of Object.entries(config.mats_prices)) {
    mats_prices[region as MarketRegions] = columns_to_saved(columns);
  }

  const shard_infos: SavedConfig["shard_infos"] = {};
  for (const [region, info] of Object.entries(config.shard_infos)) {
    shard_infos[region as MarketRegions] = shard_info_to_saved(info);
  }

  const roster_mats_owned: SavedConfig["roster_mats_owned"] = {};
  const tradable_mats_owned: SavedConfig["tradable_mats_owned"] = {};
  for (const roster_id of Object.keys(config.roster_mats_owned)) {
    roster_mats_owned[roster_id] = columns_to_saved(
      config.roster_mats_owned[roster_id],
    );
    tradable_mats_owned[roster_id] = columns_to_saved(
      config.tradable_mats_owned[roster_id],
    );
  }

  return {
    profiles: config.profiles.map(profile_to_saved),
    active_profile_index: config.active_profile_index,
    last_seen_version: config.last_seen_version,
    mats_prices,
    roster_mats_owned,
    tradable_mats_owned,
    all_regions: { ...config.all_regions },
    shard_infos,
    latest_market_data: config.latest_market_data,
    cumulative_graph: config.cumulative_graph,
    show_all_rows: config.show_all_rows,
    auto_deduct_costs: config.auto_deduct_costs,
    auto_fetch: config.auto_fetch,
    enabled_annotations: config.enabled_annotations,
  };
}

export function from_saved(saved: Partial<SavedConfig>): RosterConfig {
  const out = default_roster_config();

  for (const region of Object.keys(out.mats_prices) as MarketRegions[]) {
    out.mats_prices[region] = columns_from_saved(
      saved.mats_prices?.[region],
      out.mats_prices[region],
    );
    out.shard_infos[region] = shard_info_from_saved(
      saved.shard_infos?.[region],
    );
  }

  const roster_ids = Object.keys(saved.roster_mats_owned ?? {}).map(Number);
  if (roster_ids.length > 0) {
    const templates = out.roster_mats_owned[0];
    out.roster_mats_owned = {};
    out.tradable_mats_owned = {};
    out.all_regions = {};
    for (const roster_id of roster_ids) {
      out.roster_mats_owned[roster_id] = columns_from_saved(
        saved.roster_mats_owned?.[roster_id],
        templates,
      );
      out.tradable_mats_owned[roster_id] = columns_from_saved(
        saved.tradable_mats_owned?.[roster_id],
        templates,
      );
      out.all_regions[roster_id] = saved.all_regions?.[roster_id] ?? "nae";
    }
  }

  const profiles: CharProfile[] = [];
  const saved_profiles =
    saved.profiles && saved.profiles.length > 0 ? saved.profiles : [{}];
  saved_profiles.forEach((profile, index) => {
    profiles.push(
      profile_from_saved(
        profile,
        index,
        profiles,
        Object.keys(out.roster_mats_owned).map(Number),
      ),
    );
  });
  out.profiles = profiles;

  out.active_profile_index = Math.max(
    0,
    Math.min(profiles.length - 1, saved.active_profile_index ?? 0),
  );
  out.last_seen_version = saved.last_seen_version ?? out.last_seen_version;
  out.latest_market_data = saved.latest_market_data ?? {};
  out.cumulative_graph = saved.cumulative_graph ?? out.cumulative_graph;
  out.show_all_rows = saved.show_all_rows ?? out.show_all_rows;
  out.auto_deduct_costs = saved.auto_deduct_costs ?? out.auto_deduct_costs;
  out.auto_fetch = saved.auto_fetch ?? out.auto_fetch;
  if (
    Array.isArray(saved.enabled_annotations) &&
    saved.enabled_annotations.length === out.enabled_annotations.length
  ) {
    out.enabled_annotations = saved.enabled_annotations;
  }

  return out;
}
