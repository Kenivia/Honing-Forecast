import { DEFAULT_ARTISAN_MULTIPLIER } from "./Constants";
import { Upgrade } from "./KeyedUpgrades";

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
    let min_count = Math.min(count, 10);

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
  return (Math.min(artisan, 1) * 100).toFixed(2);
}
export function cumulative_chance(
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
  let artisan = 0;
  let cum_chance = 1.0;
  // console.log(upgrade.normal_dist, extra_arr, upgrade.state)

  for (let count = 0; count < total_count; count++) {
    let min_count = Math.min(count, 10);

    let current_chance = Math.min(
      1,
      upgrade.base_chance +
        upgrade.extra_chance +
        min_count * upgrade.base_chance * 0.1 +
        extra_arr[count],
    );
    if (artisan >= 1.0) {
      return (100.0).toFixed(2);
    }
    cum_chance *= 1 - current_chance;
    artisan +=
      DEFAULT_ARTISAN_MULTIPLIER * current_chance * upgrade.artisan_rate;
    if (current_chance == 1.0) {
      return (100.0).toFixed(2);
    }
  }

  return (Math.max(1 - cum_chance, 0) * 100).toFixed(2);
}
