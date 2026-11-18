// Serca vambrace normal honing for the info page demo, run through the real optimizer.
import SERCA from "../../../../crates/core/src/constants/Serca Oct 2026.json";
import {
  ALL_LABELS,
  NUM_BASE_MATS,
  default_selected_bundles,
  RATE_SCALE,
} from "@/Utils/Constants";
import { DEFAULT_CHAR_PROFILE } from "@/Stores/CharacterProfile";
import { payload_for } from "@/WasmInterface/PayloadBuilder";
import { to_upgrade_key } from "@/Utils/KeyedUpgrades";
import { MAX_AMOUNT } from "@/Utils/InputColumn";

const TIER = 1;
const VAMBRACE = 2;
const VAMBRACE_PIECE = 6;
const FUSION = 4;
export const FUSION_LABEL = "Serca Fusion";
// NA prices of 2026-11-18, but fusion inflated so the optimizer juices even with none owned
export const PRICES: Record<string, number> = {
  [FUSION_LABEL]: 800,
  "Glacier's Breath": 285,
  "Lava's Breath": 394,
};
// by bundle key; T4 fusion too, or converting it would undercut the inflated price
const BUNDLE_PRICES = {
  ...Object.fromEntries(
    Object.entries(PRICES).map(([label, p]) => [`${label} x1`, p]),
  ),
  "Fusion x1": PRICES[FUSION_LABEL],
};

export const upgrade_name = (upgrade_index: number) =>
  `Vambrace +${upgrade_index + 1}`;

export const fusion_per_tap = (upgrade_index: number) =>
  SERCA.NORMAL_VAMBRACE_COST[FUSION][upgrade_index];

// the juices (not books) an upgrade can take, as [id, rate, amount per tap]
const juice_rows = (upgrade_index: number) =>
  SERCA.JUICE_BOOKS_AVAIL.filter(
    (r) =>
      r[0] <= 1 &&
      r[1] === 0 &&
      r[2] === VAMBRACE &&
      r[3] === upgrade_index + 1,
  ).map((r) => [r[0], r[4], r[5]]);

// Pity with nothing (or every juice) used, in the integers of Rust's probability_distribution.
export function max_taps(upgrade_index: number, juiced = false): number {
  const extra = juiced
    ? juice_rows(upgrade_index).reduce((s, [, rate]) => s + rate, 0)
    : 0;
  // the game's tables are out of 10,000
  const at = (table: number[][]) =>
    table[VAMBRACE][upgrade_index] * (RATE_SCALE / 10000);
  const base = at(SERCA.NORMAL_CHANCES);
  const bonus = at(SERCA.NORMAL_FAIL_BONUS);
  const bonus_max = at(SERCA.NORMAL_FAIL_BONUS_MAX);
  const threshold = at(SERCA.ARTISAN_THRESHOLD);
  let artisan = 0;
  for (let count = 0; ; count++) {
    if (artisan >= threshold) return count + 1;
    const rate = Math.min(
      RATE_SCALE,
      base + Math.min(count * bonus, bonus_max) + extra,
    );
    artisan += rate;
    if (rate === RATE_SCALE) return count + 1;
  }
}

// Each juice the upgrades take, with the most used: every tap but the pity juiced.
export function demo_juices(levels: number[]) {
  const max: Record<number, number> = {};
  for (const l of levels)
    for (const [id, , amt] of juice_rows(l))
      max[id] = (max[id] ?? 0) + amt * (max_taps(l, true) - 1);
  return Object.entries(max).map(([id, most]) => ({
    label: ALL_LABELS[TIER][NUM_BASE_MATS + +id],
    max: most,
  }));
}

// A fresh Serca profile with just these upgrades, at streak length 1.
export function demo_profile(levels: number[]) {
  const profile = structuredClone(DEFAULT_CHAR_PROFILE);
  profile.tier = TIER;
  profile.min_resolution = 1;
  // only fusion and the juices are traded; everything else is disabled, so free
  const juices = ALL_LABELS[TIER].slice(NUM_BASE_MATS);
  for (const label of Object.keys(profile.bound_budgets.enabled))
    profile.bound_budgets.enabled[label] =
      label === FUSION_LABEL || juices.includes(label);
  profile.keyed_upgrades = Object.fromEntries(
    levels.map((upgrade_index) => [
      to_upgrade_key(VAMBRACE_PIECE, upgrade_index, true, TIER),
      {
        piece_index: VAMBRACE_PIECE,
        upgrade_index,
        is_normal_honing: true,
        starting_artisan: 0,
        starting_num_taps: 0,
        state: null,
        unlocked: false,
        adv_progress: null,
        expanded: false,
        taps_since_last_input: 0,
        used_materials: null,
        double_balls: false,
      },
    ]),
  );
  return profile;
}

// Nothing owned but bound fusion and juice (by label), at PRICES; the rest is endless.
export function demo_payload(
  profile: ReturnType<typeof demo_profile>,
  owned: Record<string, number>,
) {
  const bound: Record<string, number> = { ...owned };
  for (const [label, on] of Object.entries(profile.bound_budgets.enabled))
    if (on === false) bound[label] = MAX_AMOUNT;
  return payload_for(
    profile,
    bound,
    {},
    {},
    {},
    [],
    BUNDLE_PRICES,
    default_selected_bundles(),
  );
}
