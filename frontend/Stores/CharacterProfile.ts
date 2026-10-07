import {
  ADV_COLS,
  ALL_MATERIAL_LABELS,
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
import {
  OptimizerOverride,
  AdvOverride,
  NormalOverride,
} from "@/WasmInterface/PayloadBuilder";

export interface CharProfile {
  roster_id: number;

  optimizer_treatment_plan: TreatmentPlan;
  histogram_treatment_plan: TreatmentPlan;
  express_event: boolean;
  char_name: string;

  normal_grid: StatusGrid;
  adv_grid: StatusGrid;

  keyed_upgrades: KeyedUpgrades; // see Interface for the definition of these

  // One column each, keyed by material label and covering every tier. A material shared
  // between tiers therefore has one value, not one per tier.
  special_budget: InputColumn;
  bound_budgets: InputColumn;
  leftover_price: InputColumn;

  auto_start_optimizer: boolean;
  tier: number;
  min_resolution: number; // currently not used (always 1)
  num_threads: number; // currently not used (always 1)
  metric_type: number; // currently not used (always 1)
  optimizer_override: OptimizerOverride;

  lock_fetched_done: boolean;
  pretend_30_40_x2_grace: boolean;
}

export enum TreatmentPlan {
  // this also serves as the index to chances_arr so order matters here
  TreatRosterAsTradable,
  TreatRosterAsBound,
  TreatTradableAsBound,
  TreatAllAsTradable,
}

export const DEFAULT_CHAR_PROFILE: CharProfile = {
  optimizer_treatment_plan: TreatmentPlan.TreatRosterAsBound,
  histogram_treatment_plan: TreatmentPlan.TreatRosterAsTradable,
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
  // implicit 0 leftover here, currently UI does not allow changing this
  leftover_price: create_input_column(InputType.Int, ALL_MATERIAL_LABELS),

  tier: 0,
  min_resolution: 1,
  num_threads: 1,
  metric_type: 1,

  roster_id: 0,
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
  return structuredClone(DEFAULT_CHAR_PROFILE);
}
