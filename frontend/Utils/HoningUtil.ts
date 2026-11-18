import { CharProfile } from "@/Stores/CharacterProfile";
import { NUM_ADV_PIECES } from "./Constants";
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

// Artisan after this many taps of the upgrade's state, as Rust worked it out (a fraction, may pass 1).
export function artisan_number(upgrade: Upgrade, taps: number): number {
  const trail = upgrade.tap_artisan;
  return trail[clamp(0, taps, trail.length - 1)];
}
export function artisan_string(upgrade: Upgrade, taps: number): string {
  return locale_to_fixed(Math.min(artisan_number(upgrade, taps), 1) * 100, 2, true);
}

// Chance in percent after `failed` failed taps with nothing used.
export function chance_percent(upgrade: Upgrade, failed: number): number {
  const chances = upgrade.fail_chances;
  return chances[clamp(0, failed, chances.length - 1)] * 100;
}
// The failed tap count nearest a chance in percent.
export function chance_to_taps(upgrade: Upgrade, percent: number): number {
  const gaps = upgrade.fail_chances.map((c) => Math.abs(c * 100 - percent));
  return gaps.indexOf(Math.min(...gaps));
}
