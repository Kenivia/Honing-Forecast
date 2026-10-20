import { CharProfile } from "@/Stores/CharacterProfile";
import { DEFAULT_ARTISAN_MULTIPLIER, NUM_ADV_PIECES } from "./Constants";
import { clamp, locale_to_fixed } from "./Helpers";
import { Upgrade, UpgradeStatus } from "./KeyedUpgrades";

export function ilevel(
  profile: CharProfile,
  mode: "achieved" | "pending",
): string {
  // console.log(profile.normal_grid, "start");
  const done = (value: UpgradeStatus) =>
    value === UpgradeStatus.Done ||
    value === UpgradeStatus.FetchedDone ||
    (mode === "pending" && value === UpgradeStatus.Want);

  let out = profile.tier === 0 ? 1590 : 1635;
  if (profile.tier === 0) {
    for (let row = 0; row < NUM_ADV_PIECES; row++) {
      const highest_plus = profile.adv_grid[row].findLastIndex(done) + 1;
      out += (highest_plus * 10) / 6;
    }
  } else {
    out += 40;
  }
  for (let row = 0; row < NUM_ADV_PIECES; row++) {
    const highest_plus = profile.normal_grid[row].findLastIndex(done) + 1;
    // console.log(profile.normal_grid[row], highest_plus, row);
    if (highest_plus === 0) {
      return "?";
    } else {
      out += (highest_plus * 5) / 6;
    }
  }
  return locale_to_fixed(out, 2);
}

export function artisan_number(
  upgrade: Upgrade,
  total_count: number,
  juice_info: any,
): number {
  let extra_arr = upgrade.state.slice(0, total_count).map((ids) => {
    let chance = 0.0;
    for (const id of ids) {
      chance += juice_info.all_juices[id][upgrade.piece_type_usize].get(
        upgrade.upgrade_index,
      ).normal_chance;
    }

    return chance;
  });
  let artisan = upgrade.starting_artisan;
  for (let count = 0; count < total_count; count++) {
    let min_count = Math.min(count + upgrade.starting_num_taps, 10);

    let current_chance = Math.min(
      1,
      upgrade.base_chance +
        upgrade.extra_chance +
        min_count * upgrade.base_chance * 0.1 +
        // a tap past the state is unjuiced, as in Rust
        (extra_arr[count] ?? 0),
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

  return artisan;
}
export function artisan_string(
  upgrade: Upgrade,
  total_count: number,
  juice_info: any,
): string {
  return locale_to_fixed(
    Math.min(artisan_number(upgrade, total_count, juice_info), 1) * 100,
    2,
    true,
  );
}

// Base chance in percent after `failed` failed taps: +10% of base per failure, up to 10.
export function chance_percent(upgrade: Upgrade, failed: number): number {
  const steps = Math.min(10, failed);
  return Math.min(
    100,
    (upgrade.base_chance * (1 + steps / 10) + upgrade.extra_chance) * 100,
  );
}
// The failed tap count nearest a base chance in percent.
export function chance_to_taps(upgrade: Upgrade, percent: number): number {
  const steps =
    (percent - (upgrade.base_chance + upgrade.extra_chance) * 100) /
    (upgrade.base_chance * 10);
  return clamp(0, Math.round(steps), 10);
}
