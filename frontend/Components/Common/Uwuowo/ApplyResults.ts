import { CharProfile } from "@/Stores/CharacterProfile";
import { UwuowoPiece } from "./UwuowoUtils";
import {
  apply_done_want,
  change_tier,
  convert_apply_done_want,
} from "@/Components/Character/StatusInput/StatusInputUtil";
import { ADV_COLS, NUM_PIECES } from "@/Utils/Constants";
import {
  grids_to_keyed,
  to_upgrade_key,
  UpgradeStatus,
} from "@/Utils/KeyedUpgrades";
import { grid_change_callback } from "@/Components/Character/CharWorkerUtils";

export function apply_results(
  target_profile: CharProfile,
  results: UwuowoPiece[],
  force_t4: boolean,
  dont_run: boolean,
) {
  const old_tier: number = target_profile.tier;
  const new_tier: number = force_t4
    ? 0
    : results.some((x) => x.tier === 1)
      ? 1
      : 0;
  // console.log(results, old_tier, new_tier);
  if (old_tier !== new_tier) {
    change_tier(target_profile, true);
  }

  const want_adv: { row: number; upgrade_index: number; diff: number }[] = [];
  for (let row = 0; row < NUM_PIECES; row++) {
    const want =
      results[row].tier === old_tier
        ? target_profile.normal_grid[row].findLastIndex(
            (value) => value == UpgradeStatus.Want,
          ) + 1
        : 0;
    if (results[row].tier !== new_tier) {
      // console.log(results[row].plus_n, want, results[row].tier, new_tier);
      convert_apply_done_want(
        new_tier === 1 ? 0 : 1, // the function expects actual changing, old_tier might not equal resuelts[row].tierw
        new_tier,
        force_t4 ? 18 : results[row].plus_n,
        want,
        target_profile.normal_grid[row],
        true,
      );
      //   convert_apply_done_want(
      //     old_tier,
      //     new_tier,
      //     results[row].plus_n,
      //     0, // remove all pending if tier changed
      //     target_profile.normal_grid[row],
      //   );
    } else {
      // console.log(
      //   results[row].plus_n,
      //   want,
      //   results[row].tier,
      //   old_tier,
      //   target_profile.normal_grid[row],
      // );
      apply_done_want(
        results[row].plus_n,
        want,
        target_profile.normal_grid[row],
        true,
      );
    }

    if (new_tier === 0) {
      const upper =
        results[row].tier == 0 ? Math.floor(results[row].adv / 10) : 4;
      for (let col = 0; col < upper; col++) {
        target_profile.adv_grid[row][col] = UpgradeStatus.Done;
      }
      let non_done_start = upper;

      if (results[row].adv - Math.floor(results[row].adv / 10) * 10 > 0) {
        target_profile.adv_grid[row][upper] = UpgradeStatus.Want;
        want_adv.push({
          row,
          upgrade_index: upper,
          diff: results[row].adv - Math.floor(results[row].adv / 10),
        });
        non_done_start += 1;
      }

      for (let col = non_done_start; col < ADV_COLS; col++) {
        target_profile.adv_grid[row][col] =
          target_profile.adv_grid[row][col] === UpgradeStatus.Want
            ? UpgradeStatus.Want
            : UpgradeStatus.NotYet;
      }
    }
  }
  if (new_tier == 1) {
    for (let row = 0; row < NUM_PIECES; row++) {
      for (let col = 0; col < ADV_COLS; col++) {
        target_profile.adv_grid[row][col] = UpgradeStatus.Done;
      }
    }
  } else {
    target_profile.keyed_upgrades = grids_to_keyed(
      target_profile.normal_grid,
      target_profile.adv_grid,
      target_profile.keyed_upgrades,
      target_profile.tier,
    );
    // console.log(target_profile.keyed_upgrades);
    for (let index = 0; index < want_adv.length; index++) {
      const key = to_upgrade_key(
        want_adv[index].row,
        want_adv[index].upgrade_index,
        false,
        0,
      );
      target_profile.keyed_upgrades[key].adv_progress[0] = want_adv[index].diff;
    }
  }

  grid_change_callback(dont_run);
}
