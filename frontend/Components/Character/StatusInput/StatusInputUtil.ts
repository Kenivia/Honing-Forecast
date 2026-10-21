import {
  ADV_COLS,
  ALL_LABELS,
  NORMAL_COLS,
  NORMAL_FLOOR,
  NUM_ADV_PIECES,
  PLUS_TIER_CONVERSION,
} from "@/Utils/Constants";
import {
  check_adv_all_done,
  check_all_plus_20,
  check_revert_ilevel_ok,
} from "@/Utils/Helpers";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";

import { grid_change_callback } from "../CharWorkerUtils";
import { CharProfile } from "@/Stores/CharacterProfile";
import { useRuntimeStore } from "@/Stores/RuntimeState";

// what the tier button says, on it, on hover, and whether converting resets anything
export const tier_label = (profile: CharProfile) =>
  profile.tier == 0 ? "Convert to T4.5 Serca" : "Revert back to T4";
export const tier_warning = (profile: CharProfile) =>
  profile.tier == 0 &&
  !(check_all_plus_20(profile) && check_adv_all_done(profile));
export function tier_tooltip(profile: CharProfile) {
  if (profile.tier == 0) {
    if (!tier_warning(profile))
      return "Eligible for conversion to T4.5 Serca gear";
    return (
      "Warning! " +
      [
        !check_adv_all_done(profile)
          ? "All Adv honing will be set to +40"
          : null,
        !check_all_plus_20(profile)
          ? "All Gear will be set to +11 (Serca)"
          : null,
      ]
        .filter((x) => x !== null)
        .join(", \n")
    );
  }
  return check_revert_ilevel_ok(profile) === true
    ? "Can go back to T4"
    : "Cannot convert back to T4 because +" +
        String(check_revert_ilevel_ok(profile)) +
        " cannot be mapped directly to a T4 upgrade";
}

export function change_tier(target_profile: CharProfile, fetched?: boolean) {
  let old_tier = target_profile.tier;
  if (check_revert_ilevel_ok(target_profile) === true || fetched) {
    target_profile.tier = target_profile.tier == 0 ? 1 : 0;
  }
  let new_tier = target_profile.tier;

  if (
    (new_tier === null || old_tier === null || new_tier == old_tier) &&
    !fetched
  )
    return;
  if (ALL_LABELS.length != 2) {
    // the upgrade-grid remap assumes two tiers
    throw new Error(
      "grid conversion between more than 2 tiers not implemented",
    );
  }

  // may be a non-active character during a roster import, hence the lookup by id
  const target_runtime = useRuntimeStore().for_char(target_profile.id);
  target_runtime?.optimizer.cancel_and_clear_prev_result();
  target_runtime?.histogram.cancel_and_clear_prev_result();
  target_runtime?.marginal.cancel_and_clear_prev_result();

  // Budgets are left as they are: a Serca character's T4 stock is counted as Serca when the
  // payload is built (merge_lower_tier).

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

// The row's +n done and wanted in the old tier, as the new tier's. A level with no equal takes
// the one below it, and below the old floor is the floor. Levels fetched in the old tier stay
// fetched, and so does the new tier's floor.
export function convert_apply_done_want(
  old_tier: number,
  new_tier: number,
  done_plus_n: number,
  want_plus_n: number,
  row: UpgradeStatus[],
  fetched?: boolean,
) {
  const conv = PLUS_TIER_CONVERSION[old_tier];
  const floor = Math.min(...Object.keys(conv).map(Number));
  const to_new = (plus_n: number) => {
    const n = Math.max(floor, plus_n);
    return conv[String(n)] ?? conv[String(n - 1)] ?? 25;
  };
  const fetched_n =
    row.findLastIndex((value) => value == UpgradeStatus.FetchedDone) + 1;
  apply_done_want(
    to_new(done_plus_n),
    to_new(want_plus_n),
    row,
    fetched,
    fetched_n >= floor ? to_new(fetched_n) : NORMAL_FLOOR[new_tier],
  );
}

// Done below +done (fetched below +fetched_below, or all if fetched), then Want below +want.
export function apply_done_want(
  done_plus_n: number,
  want_plus_n: number,
  row: UpgradeStatus[],
  fetched?: boolean,
  fetched_below = 0,
) {
  for (let col = 0; col < NORMAL_COLS; col++) {
    row[col] =
      col < done_plus_n
        ? fetched || col < fetched_below
          ? UpgradeStatus.FetchedDone
          : UpgradeStatus.Done
        : col < want_plus_n
          ? UpgradeStatus.Want
          : UpgradeStatus.NotYet;
  }
}
