import { CharProfile } from "@/Stores/CharacterProfile";
import {
  ADV_COLS,
  IconMap,
  NUM_PIECES,
  PIECE_NAMES,
  PLUS_TIER_CONVERSION,
} from "./Constants";

import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { Upgrade, UpgradeStatus } from "./KeyedUpgrades";
import { LOCALE_DECIMAL, parse_locale_float } from "./InputColumn";

export function locale_to_fixed(
  inp: number,
  place?: number,
  round_down?: boolean,
) {
  // console.log(inp, Math.floor(inp * Math.pow(10, place ? place : 0)));
  return (
    round_down
      ? Math.floor(inp * Math.pow(10, place ? place : 0)) /
        Math.pow(10, place ? place : 0)
      : inp
  )
    .toFixed(place)
    .replace(".", LOCALE_DECIMAL);
}

// ideally i would merge this with the inputcolumn parsing stuff cos it's the same logic
// but like the inputcolumn stuff is so intertwined with inputcolumn meta data
// so i can't really be bothered
export function clean_percentage_input(
  input: string,
  fallback?: number,
  round_down?: boolean,
): string {
  const cleaned = input.toString().replace(/[^\d,.]/g, "");
  return !isFinite(parse_locale_float(cleaned))
    ? locale_to_fixed(fallback, 2, round_down)
    : clamp_percentage(cleaned, round_down);
}
export function clamp_percentage(input: string, round_down?: boolean): string {
  return locale_to_fixed(
    clamp(0, parse_locale_float(input), 100),
    2,
    round_down,
  );
}
export function clamp(min: number, input: number, max: number): number {
  return Math.max(min, Math.min(max, input));
}

export function has_upgrades_in_range(
  low: number,
  high: number,
  is_weapon: boolean,
  is_adv: boolean,
) {
  // inclusive

  const { active_profile } = storeToRefs(useRosterStore());

  for (
    let row = is_weapon ? 5 : 0;
    row < (is_weapon ? NUM_PIECES : NUM_PIECES - 1);
    row++
  ) {
    for (let col = low - 1; col < high; col++) {
      if (
        (is_adv
          ? active_profile.value.adv_grid
          : active_profile.value.normal_grid)[row][col] === UpgradeStatus.Want
      ) {
        return true;
      }
    }
  }

  return false;
}
export function format_char_name(
  raw: string,
  char_index: number,
  profiles?: CharProfile[],
): string {
  const all_profiles =
    profiles === null || profiles === undefined
      ? storeToRefs(useRosterStore()).all_profiles.value
      : profiles;

  let original = raw.replace(/ /g, ""); //
  // 2. Remove non-alphanumeric (keep underscores)
  original = original.replace(/[^\p{L}\p{N}_]/gu, "");
  // 3. Lowercase every letter after the first
  original = original
    .replace(/(?<=.)[A-Z]/g, (c) => c.toLowerCase())
    .slice(0, 16);

  const new_string =
    String(original).charAt(0).toUpperCase() + String(original).slice(1);
  // 4. If empty, or already taken by another profile, append index
  const otherNames = all_profiles
    .filter((_, i) => i !== char_index)
    .map((x) => x.char_name);
  let dup_count = 1;
  let result = new_string;
  while (!result || otherNames.includes(result)) {
    result = new_string + String(dup_count);
    dup_count += 1;
  }
  return result;
}

export function check_adv_all_done() {
  const { active_profile } = storeToRefs(useRosterStore());
  for (let row = 0; row < NUM_PIECES; row++) {
    for (let col = 0; col < ADV_COLS; col++) {
      if (active_profile.value.adv_grid[row][col] != UpgradeStatus.Done) {
        return false;
      }
    }
  }
  return true;
}
export function check_all_plus_20(): number | boolean {
  const { active_profile } = storeToRefs(useRosterStore());
  for (let row = 0; row < NUM_PIECES; row++) {
    let highest_done =
      active_profile.value.normal_grid[row].findLastIndex(
        (value) => value == UpgradeStatus.Done,
      ) + 1;

    if (!(highest_done >= 20)) {
      return false;
    }
  }
  return true;
}
export function check_revert_ilevel_ok(): number | boolean {
  const { active_profile } = storeToRefs(useRosterStore());
  if (active_profile.value.tier == 0) {
    return true;
  }
  for (let row = 0; row < NUM_PIECES; row++) {
    let highest_done =
      active_profile.value.normal_grid[row].findLastIndex(
        (value) => value == UpgradeStatus.Done,
      ) + 1;

    if (
      !Object.hasOwn(
        PLUS_TIER_CONVERSION[active_profile.value.tier],
        String(highest_done),
      )
    ) {
      return highest_done;
    }
    let highest_want =
      active_profile.value.normal_grid[row].findLastIndex(
        (value) => value == UpgradeStatus.Done || value == UpgradeStatus.Want,
      ) + 1;

    if (
      !Object.hasOwn(
        PLUS_TIER_CONVERSION[active_profile.value.tier],
        String(highest_want),
      )
    ) {
      return highest_want;
    }
  }
  return true;
}
// export function check_revert_eligibility(): boolean {
//     return check_adv_all_done() && check_revert_ilevel_ok() === true
// }

export function cssVar(name: string, fallback: string) {
  if (typeof window === "undefined") return fallback;
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
  return value || fallback;
}

export function debounce<T extends (...args: any[]) => void>(
  fn: T,
  delay: number,
): T {
  let timer: ReturnType<typeof setTimeout>;
  return ((...args: any[]) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  }) as T;
}

export function metric_to_text(metric: number | null | undefined) {
  if (metric === null || metric === undefined || !Number.isFinite(metric))
    return "N/A";
  return `${Math.round(metric == 0 ? metric : -metric).toLocaleString("en-US")}`;
}
export function get_icon_path(name: string) {
  return IconMap[name] ?? "";
}
export function get_piece_name(upgrade: Upgrade) {
  return PIECE_NAMES[upgrade.piece_type];
}

const ordinalRules = new Intl.PluralRules("en", { type: "ordinal" });
export function toOrdinal(n: number): string {
  const suffixes: Record<Intl.LDMLPluralRule, string> = {
    zero: "th",
    one: "st",
    two: "nd",
    few: "rd",
    many: "th",
    other: "th",
  };

  return `${n}${suffixes[ordinalRules.select(n)]}`;
}
