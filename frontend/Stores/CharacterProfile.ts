import type { SnapshotResult } from "@/Utils/Scans";
import {
  ADV_COLS,
  ALL_MATERIAL_LABELS,
  BandPlan,
  MAIN_BANDS,
  NORMAL_COLS,
  NUM_ADV_PIECES,
  NUM_PIECES,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import {
  create_input_column,
  InputColumn,
  InputType,
} from "@/Utils/InputColumn";
import { create_status_grid } from "@/Utils/StatusGrid";
import { KeyedUpgrades, StatusGrid } from "@/Utils/KeyedUpgrades";
import type { ManifestChest } from "@/Utils/Chest";
import type { SourcesConfig } from "@/Utils/IncomeSources";
import type { CachedState } from "@/WasmInterface/PayloadBuilder";
import {
  OptimizerOverride,
  AdvOverride,
  NormalOverride,
} from "@/WasmInterface/Overrides";

export interface CharProfile {
  // stable and unique, what everything but the route identifies a character by
  id: string;
  roster_id: string;

  // what a leftover unit of each ownership band is worth
  band_values: BandPlan;
  // which cumulative ownership band the chance column and graph annotation show, 0 to 2
  chance_band: number;
  express_event: boolean;
  char_name: string;

  normal_grid: StatusGrid;
  adv_grid: StatusGrid;

  keyed_upgrades: KeyedUpgrades; // see Interface for the definition of these

  // One column each, keyed by material label and covering every tier. A material shared
  // between tiers therefore has one value, not one per tier.
  special_budget: InputColumn;
  bound_budgets: InputColumn;
  // chests still to be opened, as the scanner's manifest exported them
  chests: ManifestChest[];
  // keys of chests, its own or the roster's, this character may not open
  forbidden_chests: string[];
  // display name from lostark.bible ("Slayer"), "" until fetched
  class_name: string;
  // income per week typed by hand, char bound and free taps (under the special leap label)
  income_manual: boolean;
  // income per week from the sources configured on the Income page, when not typed by hand
  income_auto: boolean;
  income_sources: Partial<SourcesConfig>;
  bound_income: Record<string, number>;
  // the Income page's now: the time its budgets (a hash) were first seen, and the runs of now
  // and the weeks ahead by payload hash
  income_now: { budgets: string; at: number } | null;
  income_results: Record<string, SnapshotResult>;
  // the difficulty picked by hand per raid on the Choices page
  choice_levels: Record<string, string>;
  // the last finished optimizer run of the calculator's payload (the calculator's or the Choices
  // page's run without a choice), restored in place of a run of the same payload
  optimizer_cache: CachedState | null;
  // the Choices page's runs, [gold, metric] by payload hash
  choice_results: Record<string, [number, number]>;

  auto_start_optimizer: boolean;
  tier: number;
  min_resolution: number; // streak length, 1 or 10
  optimizer_override: OptimizerOverride;

  lock_fetched_done: boolean;
  pretend_30_40_x2_grace: boolean;
}

export const DEFAULT_CHAR_PROFILE: CharProfile = {
  id: "",
  band_values: [...MAIN_BANDS],
  chance_band: 0,
  express_event: false,
  char_name: "Newchar",

  auto_start_optimizer: true,

  normal_grid: create_status_grid(NUM_PIECES, NORMAL_COLS, 0, false),
  adv_grid: create_status_grid(NUM_ADV_PIECES, ADV_COLS, 0, true),

  keyed_upgrades: {},
  special_budget: create_input_column(InputType.Int, SPECIAL_LEAP_LABELS, {
    upper_bound: () => 33333,
  }),

  bound_budgets: create_input_column(InputType.Int, ALL_MATERIAL_LABELS, {
    enabled: (label) => label !== "Shards",
  }),
  chests: [],
  forbidden_chests: [],
  class_name: "",
  income_manual: false,
  income_auto: false,
  income_sources: {},
  bound_income: {},
  income_now: null,
  income_results: {},
  choice_levels: {},
  optimizer_cache: null,
  choice_results: {},
  tier: 0,
  min_resolution: 10,

  roster_id: "",
  optimizer_override: {
    normal: {
      juice: NormalOverride.Optimizer,
      book: NormalOverride.Optimizer,
    },
    special: {
      optimizer: true,
      weapon_first: true,
      highest_first: true,
    },
    advanced: {
      juice: AdvOverride.Optimizer,
      scroll: AdvOverride.Optimizer,
    },
  },
  lock_fetched_done: true,

  pretend_30_40_x2_grace: false,
};

export function new_char_profile(): CharProfile {
  return { ...structuredClone(DEFAULT_CHAR_PROFILE), id: crypto.randomUUID() };
}
