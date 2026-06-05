import {
  ADV_COLS,
  ALL_LABELS,
  NORMAL_COLS,
  NUM_PIECES,
  PLUS_TIER_CONVERSION,
} from "@/Utils/Constants";
import { check_revert_ilevel_ok } from "@/Utils/Helpers";
import { input_column_to_num, parse_input } from "@/Utils/InputColumn";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";

import { grid_change_callback } from "../CharWorkerUtils";
import { CharProfile } from "@/Stores/CharacterProfile";

export function change_tier(
  target_profile: CharProfile,
  dont_convert?: boolean,
) {
  let old_tier = target_profile.tier;
  if (check_revert_ilevel_ok() === true) {
    target_profile.tier = target_profile.tier == 0 ? 1 : 0;
  }
  let new_tier = target_profile.tier;

  if (
    (new_tier === null || old_tier === null || new_tier == old_tier) &&
    !dont_convert
  )
    return;
  if (ALL_LABELS.length != 2) {
    // This doesn't work for more tiers and should be updated when more tiers comes eventually
    throw new Error("conversion between more than 2 tiers not implemented yet");
  }

  target_profile.optimizer_worker_bundle?.cancel_and_clear_prev_result();
  target_profile.histogram_worker_bundle?.cancel_and_clear_prev_result();
  // profile.value.evaluation_worker_bundle?.cancel_and_clear_prev_result()

  let num_array_old = input_column_to_num(
    target_profile.bound_budgets[old_tier],
    true,
  );

  let multiplied_indices = [0, 1, 2, 4]; // red, blue, leaps, fusion
  let multiplier = new_tier == 1 ? 0.2 : 5;
  multiplied_indices.forEach(
    (index) =>
      (target_profile.bound_budgets[new_tier].data[index] = parse_input(
        target_profile.bound_budgets[old_tier],
        index,
        String(num_array_old[index] * multiplier),
        true,
      ).toLocaleString()),
  );
  // Special leaps also multiplied
  target_profile.special_budget.data[0] = parse_input(
    target_profile.special_budget,
    0,
    String(
      input_column_to_num(target_profile.special_budget, true)[0] * multiplier,
    ),
    true,
  ).toLocaleString();

  let stay_same_indices = [3, 5, 6, 7]; // shards, gold, silver, red juice
  stay_same_indices.forEach(
    (index) =>
      (target_profile.bound_budgets[new_tier].data[index] =
        target_profile.bound_budgets[old_tier].data[index]),
  );

  // special case for blue juice
  let new_num_juice_avail = (ALL_LABELS[new_tier].length - 7) / 2;
  let new_index = 7 + new_num_juice_avail;
  let old_num_juice_avail = (ALL_LABELS[old_tier].length - 7) / 2;
  let old_index = 7 + old_num_juice_avail;
  target_profile.bound_budgets[new_tier].data[new_index] =
    target_profile.bound_budgets[old_tier].data[old_index];

  // the rest have separate values between tiers
  if (dont_convert) {
    return;
  }

  for (let row = 0; row < NUM_PIECES; row++) {
    convert_apply_done_want(
      old_tier,
      new_tier,
      target_profile.normal_grid[row].findLastIndex(
        (value) => value == UpgradeStatus.Done,
      ) + 1,
      target_profile.normal_grid[row].findLastIndex(
        (value) => value == UpgradeStatus.Want || value == UpgradeStatus.Done,
      ) + 1,
      target_profile.normal_grid[row],
    );
  }
  if (new_tier == 1) {
    for (let row = 0; row < NUM_PIECES; row++) {
      for (let col = 0; col < ADV_COLS; col++) {
        target_profile.adv_grid[row][col] = UpgradeStatus.Done;
      }
    }
  }
  // console.log("callbacked")
  grid_change_callback();
}

export function convert_apply_done_want(
  old_tier: number,
  new_tier: number,
  done_plus_n: number,
  want_plus_n: number,
  row: UpgradeStatus[],
) {
  let highest_done = Math.max(new_tier == 1 ? 20 : 11, done_plus_n);
  let highest_want = Math.max(new_tier == 1 ? 20 : 11, want_plus_n);
  let converted_done = PLUS_TIER_CONVERSION[old_tier][String(highest_done)];
  let converted_want =
    highest_want > 0
      ? PLUS_TIER_CONVERSION[old_tier][String(highest_want)]
      : converted_done;
  console.log(
    converted_done,
    converted_want,
    highest_done,
    highest_want,
    old_tier,
    new_tier,
  );
  for (let col = 0; col < NORMAL_COLS; col++) {
    if (col < converted_done) {
      row[col] = UpgradeStatus.Done;
    } else if (col < converted_want) {
      row[col] = UpgradeStatus.Want;
    } else {
      row[col] = UpgradeStatus.NotYet;
    }
  }
}

export function apply_done_want(
  converted_done: number,
  converted_want: number,
  row: UpgradeStatus[],
) {
  for (let col = 0; col < NORMAL_COLS; col++) {
    if (col < converted_done) {
      row[col] = UpgradeStatus.Done;
    } else if (col < converted_want) {
      row[col] = UpgradeStatus.Want;
    } else {
      row[col] = UpgradeStatus.NotYet;
    }
  }
}
