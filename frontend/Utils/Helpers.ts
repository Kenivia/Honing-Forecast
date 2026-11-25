import { CharProfile } from "@/Stores/CharacterProfile";
import {
  ADV_COLS,
  CLASS_KEYS,
  IconMap,
  NUM_ADV_PIECES,
  PIECE_NAMES,
  PLUS_TIER_CONVERSION,
} from "./Constants";

import ITEMS from "../../scanner_setup/templates/items.json";
import { Upgrade, UpgradeStatus } from "./KeyedUpgrades";
import { LOCALE_DECIMAL, parse_locale_float } from "./InputColumn";

export function locale_to_fixed(
  inp: number,
  place = 0,
  round_down?: boolean, // used for artisan (to match in-game)
) {
  const scale = Math.pow(10, place);
  // the epsilon undoes float error, e.g. 0.29 * 100 = 28.999...
  return (round_down ? Math.floor(inp * scale + 1e-9) / scale : inp)
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
  profile: CharProfile,
  low: number,
  high: number,
  is_weapon: boolean,
  is_adv: boolean,
) {
  // inclusive
  const grid = is_adv ? profile.adv_grid : profile.normal_grid;
  for (
    let row = is_weapon ? NUM_ADV_PIECES - 1 : 0;
    row < (is_weapon ? NUM_ADV_PIECES : NUM_ADV_PIECES - 1);
    row++
  ) {
    for (let col = low - 1; col < high; col++) {
      if (grid[row][col] === UpgradeStatus.Want) return true;
    }
  }
  return false;
}
// A typed name as stored: trimmed, first letter capital, A-Z after it lowercase. "" is unnamed.
export function clean_char_name(raw: string): string {
  const kept = raw
    .normalize("NFC")
    .trim()
    .replace(/(?<=.)[A-Z]/g, (c) => c.toLowerCase());
  return kept.charAt(0).toUpperCase() + kept.slice(1);
}

// what Lost Ark allows: 2 to 16 latin letters, accented or not
export const char_name_valid = (name: string) =>
  /^[\p{Script=Latin}\p{M}]{2,16}$/u.test(name.normalize("NFC"));
// what uploads carry: "" (the Worker's "no name") unless lostark.bible could find it
export const upload_name = (name: string) =>
  char_name_valid(name) ? name : "";
// the first reason a stored name isn't one Lost Ark allows, null if it is
export const name_problem = (name: string): string | null => {
  const n = name.normalize("NFC");
  if (n === "") return "Enter a name";
  if (/\p{N}/u.test(n)) return "Cannot contain numbers";
  if (/\s/u.test(n)) return "Cannot contain space";
  const symbol = n.match(/[^\p{L}\p{M}]/u)?.[0];
  if (symbol !== undefined) return `Cannot contain "${symbol}"`;
  if (/[^\p{Script=Latin}\p{M}]/u.test(n)) return "Latin letters only";
  if ([...n].length < 2) return "Too short (min 2)";
  if ([...n].length > 16) return "Too long (max 16)";
  return null;
};
// a name problem shown as a warning, not just a prompt
export const name_invalid = (name: string) =>
  name !== "" && name_problem(name) !== null;
// unnamed characters show as Newchar, Newchar1, ... per region, skipping names in use there
export function default_char_labels(
  profiles: CharProfile[],
  regions: Record<string, string>,
): Record<string, string> {
  const taken: Record<string, Set<string>> = {};
  for (const p of profiles)
    (taken[regions[p.roster_id]] ??= new Set()).add(p.char_name);
  const out: Record<string, string> = {};
  for (const p of profiles) {
    if (p.char_name) continue;
    const used = taken[regions[p.roster_id]];
    let label = "Newchar";
    for (let n = 1; used.has(label); n++) label = `Newchar${n}`;
    used.add(label);
    out[p.id] = label;
  }
  return out;
}

// a character's url key: the first of a label in a region keeps it, later ones take label-2, label-3, ... unused
export function route_labels(
  profiles: CharProfile[],
  regions: Record<string, string>,
  label: (p: CharProfile) => string,
): Record<string, string> {
  const used: Record<string, Set<string>> = {};
  const out: Record<string, string> = {};
  for (const p of profiles) {
    const set = (used[regions[p.roster_id]] ??= new Set());
    if (!set.has(label(p))) set.add((out[p.id] = label(p)));
  }
  for (const p of profiles) {
    if (out[p.id] !== undefined) continue;
    const set = used[regions[p.roster_id]];
    let n = 2;
    while (set.has(`${label(p)}-${n}`)) n++;
    set.add((out[p.id] = `${label(p)}-${n}`));
  }
  return out;
}

const is_done = (value: UpgradeStatus) =>
  value === UpgradeStatus.Done || value === UpgradeStatus.FetchedDone;
// the highest +n done, or done and wanted, per armour and weapon row
const highest = (profile: CharProfile, row: number, want: boolean) =>
  profile.normal_grid[row].findLastIndex(
    (value) => is_done(value) || (want && value === UpgradeStatus.Want),
  ) + 1;

export function check_adv_all_done(profile: CharProfile) {
  return profile.adv_grid
    .slice(0, NUM_ADV_PIECES)
    .every((row) => row.slice(0, ADV_COLS).every(is_done));
}
export function check_all_plus_20(profile: CharProfile): boolean {
  for (let row = 0; row < NUM_ADV_PIECES; row++) {
    if (highest(profile, row, false) < 20) return false;
  }
  return true;
}
// true, or the first +n with no T4 equal
export function check_revert_ilevel_ok(profile: CharProfile): number | true {
  if (profile.tier == 0) return true;
  for (let row = 0; row < NUM_ADV_PIECES; row++) {
    for (const want of [false, true]) {
      const plus_n = highest(profile, row, want);
      if (!Object.hasOwn(PLUS_TIER_CONVERSION[profile.tier], String(plus_n)))
        return plus_n;
    }
  }
  return true;
}

export function cssVar(name: string, fallback: string) {
  if (typeof window === "undefined") return fallback;
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
  return value || fallback;
}

// white at t = 0 to the #rrggbb colour at t = 1
export function mix(hex: string, t: number) {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  const [r, g, b] = c.map((v) => Math.round(255 + (v - 255) * t));
  return `rgb(${r},${g},${b})`;
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
  // `|| 0` turns a rounded -0 into 0
  return `${(Math.round(-metric) || 0).toLocaleString("en-US")}`;
}
// a whole percentage, with ~ where rounding hides that it is not quite 0 or 100
export function chance_text(chance: number) {
  const percent = (chance * 100).toFixed(1);
  const near =
    (percent === "0.0" && chance > 0) || (percent === "100.0" && chance < 1);
  return `${near ? "~" : ""}${percent}%`;
}
export function get_icon_path(name: string) {
  return IconMap[name] ?? "";
}
export function class_key(class_name: string) {
  return (class_name ?? "").toLowerCase().replace(/[^a-z]/g, "");
}
// The class's honed gear (scripts/game_files/equipment_icons.py), Berserker's for an unknown class; vambraces are Serca only.
export function get_piece_icon(
  piece: string,
  class_name: string,
  tier: number,
  sidereal_level: number | null = null,
) {
  const key = CLASS_KEYS.has(class_key(class_name))
    ? class_key(class_name)
    : "berserker";
  if (piece === "Weapon" && sidereal_level !== null)
    return `/Icons/Equipments/Sidereal/${key}/${sidereal_level}.webp`;
  const set = tier === 1 || piece === "Vambrace" ? "Serca" : "T4";
  return `/Icons/Equipments/${set}/${key}/${piece}.webp`;
}
// Rarity background: honed gear is ancient, the vambrace goes by its level, a sidereal weapon is sidereal.
export function get_piece_bg(piece: string, level: number, sidereal = false) {
  const rarity = sidereal
    ? "sidereal"
    : piece !== "Vambrace" || level > 20
      ? "ancient"
      : level > 15
        ? "relic"
        : level > 10
          ? "legendary"
          : "epic";
  return rarity_bg(rarity);
}
export function rarity_bg(rarity: string | null | undefined) {
  return rarity
    ? { backgroundImage: `url(/Icons/Backgrounds/${rarity}.webp)` }
    : {};
}
// Material rarity as the scanner's items.json has it; gold, silver and shards have none.
const MATERIAL_RARITY: Record<string, string> = {
  "Enhanced 19-20 Armor": "relic",
  "Enhanced 19-20 Weapon": "relic",
};
for (const item of ITEMS as any[])
  if (item.label && item.rarity) MATERIAL_RARITY[item.label] ??= item.rarity;
export function get_icon_bg(label: string) {
  return rarity_bg(MATERIAL_RARITY[label]);
}
export function vambrace_done_level(profile: CharProfile) {
  return (
    profile.normal_grid[PIECE_NAMES.indexOf("Vambrace")].findLastIndex(
      (s) => s === UpgradeStatus.Done || s === UpgradeStatus.FetchedDone,
    ) + 1
  );
}
export function get_piece_name(upgrade: Upgrade) {
  return PIECE_NAMES[upgrade.piece_index];
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

// saves text as a .json file
export function download_json(name: string, text: string) {
  const url = URL.createObjectURL(
    new Blob([text], { type: "application/json" }),
  );
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  link.click();
  URL.revokeObjectURL(url);
}
