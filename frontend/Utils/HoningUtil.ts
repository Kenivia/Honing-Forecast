import { CharProfile } from "@/Stores/CharacterProfile";
import { DEFAULT_ARTISAN_MULTIPLIER, NUM_PIECES } from "./Constants";
import { locale_to_fixed } from "./Helpers";
import { Upgrade, UpgradeStatus } from "./KeyedUpgrades";

export function ilevel(
  profile: CharProfile,
  mode: "achieved" | "pending",
): string {
  const done = (value: UpgradeStatus) =>
    value === UpgradeStatus.Done ||
    (mode === "pending" && value === UpgradeStatus.Want);

  let out = profile.tier === 0 ? 1590 : 1635;
  if (profile.tier === 0) {
    for (let row = 0; row < NUM_PIECES; row++) {
      const highest_plus = profile.adv_grid[row].findLastIndex(done) + 1;
      out += (highest_plus * 10) / 6;
    }
  } else {
    out += 40;
  }
  for (let row = 0; row < NUM_PIECES; row++) {
    const highest_plus = profile.normal_grid[row].findLastIndex(done) + 1;
    if (highest_plus === 0) {
      return "?";
    } else {
      out += (highest_plus * 5) / 6;
    }
  }
  return locale_to_fixed(out, 2);
}

export function artisan_function(
  upgrade: Upgrade,
  total_count: number,
  juice_info: any,
): string {
  let extra_arr = upgrade.state.slice(0, total_count).map(([juice, id]) => {
    let chance = 0.0;
    if (juice) {
      chance += juice_info.all_juices[0].data.get(
        String(upgrade.upgrade_index),
      ).normal_chance;
    }
    if (id > 0) {
      chance += juice_info.all_juices[id].data.get(
        String(upgrade.upgrade_index),
      ).normal_chance;
    }
    return chance;
  });
  let artisan = upgrade.starting_artisan;
  // console.log(upgrade.normal_dist, extra_arr, upgrade.state)

  for (let count = 0; count < total_count; count++) {
    let min_count = Math.min(count + upgrade.starting_num_taps, 10);

    let current_chance = Math.min(
      1,
      upgrade.base_chance +
        upgrade.extra_chance +
        min_count * upgrade.base_chance * 0.1 +
        extra_arr[count],
    );
    if (artisan >= 1.0) {
      break;
    }

    artisan +=
      DEFAULT_ARTISAN_MULTIPLIER * current_chance * upgrade.artisan_rate;
    if (current_chance == 1.0) {
      break; // for upgrades that have 100% passrate immediately or upgrades that have above 100% success rate (juicing last few taps of like +4 or something)
    }
  }
  // console.log(
  //   upgrade.base_chance,
  //   total_count,
  //   upgrade.extra_chance,
  //   upgrade.state,
  //   upgrade.normal_dist,
  // );
  return locale_to_fixed(Math.min(artisan, 1) * 100, 2, true);
}
// export function cumulative_chance(
//   upgrade: Upgrade,
//   total_count: number,
//   juice_info: any,
// ): string {
//   let extra_arr = upgrade.state.slice(0, total_count).map(([juice, id]) => {
//     let chance = 0.0;
//     if (juice) {
//       chance += juice_info.all_juices[0].data.get(
//         String(upgrade.upgrade_index),
//       ).normal_chance;
//     }
//     if (id > 0) {
//       chance += juice_info.all_juices[id].data.get(
//         String(upgrade.upgrade_index),
//       ).normal_chance;
//     }
//     return chance;
//   });
//   let artisan = 0;
//   let cum_chance = 1.0;
//   // console.log(upgrade.normal_dist, extra_arr, upgrade.state)

//   for (let count = 0; count < total_count; count++) {
//     let min_count = Math.min(count, 10);

//     let current_chance = Math.min(
//       1,
//       upgrade.base_chance +
//         upgrade.extra_chance +
//         min_count * upgrade.base_chance * 0.1 +
//         extra_arr[count],
//     );
//     if (artisan >= 1.0) {
//       return (100.0).toFixed(2);
//     }
//     cum_chance *= 1 - current_chance;
//     artisan +=
//       DEFAULT_ARTISAN_MULTIPLIER * current_chance * upgrade.artisan_rate;
//     if (current_chance == 1.0) {
//       return (100.0).toFixed(2);
//     }
//   }
//   console.log(
//     upgrade.base_chance,
//     total_count,
//     upgrade.extra_chance,
//     upgrade.state,
//     upgrade.normal_dist,
//     cum_chance,
//   );
//   return (Math.max(1 - cum_chance, 0) * 100).toFixed(2);
// }
