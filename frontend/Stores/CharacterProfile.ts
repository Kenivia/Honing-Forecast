import { create_worker_bundle } from "@/WasmInterface/WorkerBundle";
import {
  ADV_COLS,
  ALL_LABELS,
  NORMAL_COLS,
  NUM_PIECES,
  SPECIAL_LEAP_LABEL,
} from "@/Utils/Constants";
import {
  create_input_column,
  InputColumn,
  InputType,
  validate_input_column,
  validate_input_column_array,
} from "@/Utils/InputColumn";
import { create_status_grid, get_valid_status_grid } from "@/Utils/StatusGrid";
import {
  grids_to_keyed,
  KeyedUpgrades,
  StatusGrid,
  UpgradeStatus,
} from "@/Utils/KeyedUpgrades";
import {
  OptimizerOverride,
  AdvOverride,
  NormalOverride,
} from "@/WasmInterface/PayloadBuilder";
import { format_char_name } from "@/Utils/Helpers";

export interface CharProfile {
  roster_id: number;

  optimizer_treatment_plan: TreatmentPlan;
  histogram_treatment_plan: TreatmentPlan;
  express_event: boolean;
  char_name: string;

  // evaluation_worker_bundle: any
  optimizer_worker_bundle: any;
  histogram_worker_bundle: any;
  normal_grid: StatusGrid;
  adv_grid: StatusGrid;

  keyed_upgrades: KeyedUpgrades; // see Interface for the definition of these

  special_budget: InputColumn; // just a 1 cell column

  bound_budgets: InputColumn[]; // bound_budgets[tier].data[row] = "123" (data is string even though we don't directly modify it, should change this to number at some point ig)
  leftover_price: InputColumn[]; // The tier distinction is because there's different number of mats (rows) for each tier

  auto_start_optimizer: boolean;
  tier: number;
  min_resolution: number; // currently not used (always 1)
  num_threads: number; // currently not used (always 1)
  metric_type: number; // currently not used (always 1)
  optimizer_override: OptimizerOverride;

  lock_fetched_done: boolean;
}

export enum TreatmentPlan {
  // this also serves as the index to chances_arr so order matters here
  TreatRosterAsTradable,
  TreatRosterAsBound,
  TreatTradableAsBound,
  TreatAllAsTradable,
}

export const DEFAULT_CHAR_PROFILE_NO_WORKER: CharProfile = {
  optimizer_treatment_plan: TreatmentPlan.TreatRosterAsBound,
  histogram_treatment_plan: TreatmentPlan.TreatRosterAsTradable,
  express_event: false,
  char_name: "Newchar",

  auto_start_optimizer: true,

  optimizer_worker_bundle: null, // these need to be filled in with create_worker_bundle
  histogram_worker_bundle: null,

  normal_grid: create_status_grid(NUM_PIECES, NORMAL_COLS, 0, false),
  adv_grid: create_status_grid(NUM_PIECES, ADV_COLS, 0, true),

  keyed_upgrades: {},
  special_budget: create_input_column(
    InputType.Int,
    [SPECIAL_LEAP_LABEL],
    ["0"],
    [33333],
  ),

  bound_budgets: ALL_LABELS.map((this_labels) =>
    create_input_column(
      InputType.Int,
      this_labels,
      null,
      null,
      this_labels.map((_, index) => index != 3),
    ),
  ),
  leftover_price: ALL_LABELS.map((this_labels) =>
    create_input_column(InputType.Int, this_labels),
  ), // implicit 0 leftover here, currently UI does not allow changing this

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
};
// console.log(DEFAULT_CHAR_PROFILE_NO_WORKER);

// Worker bundles are not writable to string(and prolly shouldnt anyway), we re-make them on load
export function init_workers(parsed: any): CharProfile {
  delete parsed["optimizer_worker_bundle"];
  delete parsed["histogram_worker_bundle"];
  return {
    ...structuredClone(parsed),
    // evaluation_worker_bundle: createWorkerBundle(),
    optimizer_worker_bundle: create_worker_bundle(),
    histogram_worker_bundle: create_worker_bundle(),
  };
}

export function validate_char_profile(
  this_profile: any,
  out: any,
  index: number,
): CharProfile {
  let this_parsed: CharProfile = {
    ...structuredClone(DEFAULT_CHAR_PROFILE_NO_WORKER),
    ...this_profile,
  };

  this_parsed.char_name = format_char_name(
    this_parsed.char_name,
    index,
    out.profiles.slice(0, index),
  );
  validate_input_column_array(
    this_parsed.bound_budgets,
    DEFAULT_CHAR_PROFILE_NO_WORKER.bound_budgets,
  );
  validate_input_column_array(
    this_parsed.leftover_price,
    DEFAULT_CHAR_PROFILE_NO_WORKER.leftover_price,
  );
  validate_input_column(
    this_parsed.special_budget,
    DEFAULT_CHAR_PROFILE_NO_WORKER.special_budget,
  );

  this_parsed.normal_grid = get_valid_status_grid(
    this_parsed.normal_grid,
    DEFAULT_CHAR_PROFILE_NO_WORKER.normal_grid,
  ).map((row) =>
    row.map((x, index) =>
      index < (this_parsed.tier === 0 ? 10 : 11)
        ? UpgradeStatus.FetchedDone
        : x,
    ),
  );
  this_parsed.adv_grid = get_valid_status_grid(
    this_parsed.adv_grid,
    DEFAULT_CHAR_PROFILE_NO_WORKER.adv_grid,
  );

  this_parsed.keyed_upgrades = grids_to_keyed(
    this_parsed.normal_grid,
    this_parsed.adv_grid,
    this_parsed.keyed_upgrades,
    this_parsed.tier,
  );

  this_parsed.tier =
    this_parsed.tier === 0 || this_parsed.tier === 1 ? this_parsed.tier : 0;

  if (
    this_parsed.roster_id === null ||
    this_parsed.roster_id === undefined ||
    !Object.hasOwn(out.roster_mats_owned, this_parsed.roster_id)
  ) {
    this_parsed.roster_id = out.roster_mats_owned.keys()[0];
  }

  return init_workers(this_parsed);
}
