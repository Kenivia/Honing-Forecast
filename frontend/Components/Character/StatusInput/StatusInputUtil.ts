import {
  ADV_COLS,
  ALL_LABELS,
  CONVERTIBLE_MATERIALS,
  NORMAL_COLS,
  NUM_ADV_PIECES,
  NUM_PIECES,
  PLUS_TIER_CONVERSION,
} from "@/Utils/Constants";
import { check_revert_ilevel_ok } from "@/Utils/Helpers";
import { input_column_to_num, parse_input } from "@/Utils/InputColumn";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";

import { grid_change_callback } from "../CharWorkerUtils";
import { CharProfile } from "@/Stores/CharacterProfile";
import { useRuntimeStore } from "@/Stores/RuntimeState";

export function change_tier(target_profile: CharProfile, fetched?: boolean) {
  let old_tier = target_profile.tier;
  if (check_revert_ilevel_ok() === true || fetched) {
    target_profile.tier = target_profile.tier == 0 ? 1 : 0;
  }
  let new_tier = target_profile.tier;

  if (
    (new_tier === null || old_tier === null || new_tier == old_tier) &&
    !fetched
  )
    return;
  if (ALL_LABELS.length != 2) {
    // material conversion below is general, but the upgrade-grid remap is not
    throw new Error(
      "grid conversion between more than 2 tiers not implemented",
    );
  }

  // may be a non-active character during a roster import, hence the lookup by name
  const target_runtime = useRuntimeStore().for_char(target_profile.char_name);
  target_runtime?.optimizer.cancel_and_clear_prev_result();
  target_runtime?.histogram.cancel_and_clear_prev_result();

  // Convert the character's bound materials into the new tier's currency.
  //  - a material shared between tiers has one stored value, so there is nothing to do
  //  - a material that converts from a lower tier scales by its ratio, in whichever
  //    direction we are moving
  //  - anything tier-exclusive keeps its own value
  const columns = [target_profile.bound_budgets, target_profile.special_budget];
  for (const mat of CONVERTIBLE_MATERIALS) {
    const [from, to] =
      new_tier > old_tier
        ? [mat.from.label, mat.label]
        : [mat.label, mat.from.label];
    const scale = new_tier > old_tier ? 1 / mat.from.ratio : mat.from.ratio;
    for (const column of columns) {
      if (column.values[from] === undefined) {
        continue;
      }
      const old_value = input_column_to_num(column, true)[from];
      column.values[to] = parse_input(
        column,
        to,
        String(old_value * scale),
        true,
      ).toLocaleString();
    }
  }

  if (new_tier == 1) {
    for (let row = 0; row < NUM_ADV_PIECES; row++) {
      for (let col = 0; col < ADV_COLS; col++) {
        target_profile.adv_grid[row][col] = fetched
          ? UpgradeStatus.FetchedDone
          : UpgradeStatus.Done;
      }
    }
  }

  // the rest have separate values between tiers
  if (fetched) {
    return;
  }

  for (let row = 0; row < NUM_ADV_PIECES; row++) {
    convert_apply_done_want(
      old_tier,
      new_tier,
      target_profile.normal_grid[row].findLastIndex(
        (value) =>
          value == UpgradeStatus.Done || value == UpgradeStatus.FetchedDone,
      ) + 1,
      target_profile.normal_grid[row].findLastIndex(
        (value) =>
          value == UpgradeStatus.Want ||
          value == UpgradeStatus.Done ||
          value == UpgradeStatus.FetchedDone,
      ) + 1,
      target_profile.normal_grid[row],
    );
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
  fetched?: boolean,
) {
  let highest_done = Math.max(new_tier == 1 ? 20 : 11, done_plus_n);
  let highest_want = Math.max(new_tier == 1 ? 20 : 11, want_plus_n);

  let highest_fetched_done =
    row.findLastIndex((value) => value == UpgradeStatus.FetchedDone) + 1;

  let converted_fetched =
    PLUS_TIER_CONVERSION[old_tier][String(highest_fetched_done)] ??
    PLUS_TIER_CONVERSION[old_tier][String(highest_fetched_done - 1)] ??
    25;

  // these ?? should like never actually need to fire if we disallow appropriately? idk i just put them there just in case
  let converted_done =
    PLUS_TIER_CONVERSION[old_tier][String(highest_done)] ??
    PLUS_TIER_CONVERSION[old_tier][String(highest_done - 1)] ??
    25;
  let converted_want =
    highest_want > 0
      ? (PLUS_TIER_CONVERSION[old_tier][String(highest_want)] ??
        PLUS_TIER_CONVERSION[old_tier][String(highest_done - 1)] ??
        25)
      : converted_done;

  for (let col = 0; col < NORMAL_COLS; col++) {
    if (col < converted_done) {
      row[col] =
        col < converted_fetched || fetched
          ? UpgradeStatus.FetchedDone
          : UpgradeStatus.Done;
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
  fetched?: boolean,
) {
  for (let col = 0; col < NORMAL_COLS; col++) {
    if (col < converted_done) {
      row[col] = fetched ? UpgradeStatus.FetchedDone : UpgradeStatus.Done;
    } else if (col < converted_want) {
      row[col] = UpgradeStatus.Want;
    } else {
      row[col] = UpgradeStatus.NotYet;
    }
  }
}
