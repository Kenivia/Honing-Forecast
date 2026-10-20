import {
  ALL_LABELS,
  ALL_WORTHLESS,
  BandPlan,
  clamp_band_plan,
  GRACE_FIRST_N,
  JOINED_ADV_JUICE,
  NUM_ADV_PIECES,
  SPECIAL_LEAP_LABELS,
  ADV_ALWAYS_DOUBLE_BALLS,
  CONVERTIBLE_MATERIALS,
  ALL_BUNDLE_KEYS,
  unit_prices,
  effective_unit_prices,
} from "@/Utils/Constants";

import { toRaw } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  get_upgrade_map,
  KeyedUpgrades,
  OneUpgradeInput,
  Upgrade,
} from "@/Utils/KeyedUpgrades";
import { input_column_to_num } from "@/Utils/InputColumn";
import { storeToRefs } from "pinia";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { Bag, type Chest } from "@/Utils/Chest";
import type { CharProfile } from "@/Stores/CharacterProfile";

// I don't think it's possible to directly export this struct from rust to javascript because of all the vectors,
// so it's copied & pasted here
// Owned amounts and per-unit prices for one material. Mirrors Rust's OneMaterial.
export interface OneMaterial {
  bound: number;
  roster: number;
  tradable: number;
  taxed_price: number;
  market_price: number;
}

export interface Payload {
  // row order; the one place the order Rust's cost tables assume is declared
  material_labels: string[];
  materials: Record<string, OneMaterial>;
  // every plan to evaluate. Index 0 is always ALL_WORTHLESS, so the histogram always
  // reports plain gold spent alongside whatever the user picked.
  plans: BandPlan[];
  optimizer_plan: number;
  upgrade_info: OneUpgradeInput[];
  special_budget: number;
  special_state?: number[];
  // every chest stack of process_chests, by its key
  chests: Record<string, { count: number; band: number; options: Bag[] }>;
  // how many of each stack go to each option, from the last optimizer run, by the same key
  chest_states?: Map<string, number[]>;
  // what a chest's special leaps are called
  special_label: string;
  tier: number;
  express_event: boolean;
  min_resolution: number;
  num_threads: number;
  metric_type: number;
  adv_cache: any;
}

export enum NormalOverride {
  Full,
  Empty,
  Optimizer,
}
export enum AdvOverride {
  Full,
  Grace,
  Empty,
  Optimizer,
}
export interface StateOverride {
  juice: NormalOverride;
  book: NormalOverride;
}

// this kinda of have to work differently from stateoverride because weapon and highsest aren't independent
export interface SpecialOverride {
  optimizer: boolean;
  weapon_first: boolean;
  highest_first: boolean;
}

export interface AdvStateOverride {
  juice: AdvOverride;
  scroll: AdvOverride;
}
export interface OptimizerOverride {
  normal: StateOverride;
  special: SpecialOverride;
  advanced: AdvStateOverride;
}

function keyed_to_array(
  keyed_upgrades: KeyedUpgrades,
  upgrade_arr: Upgrade[] | null,
  tier: number,
  express: boolean,
  pretend_30_40: boolean,
  normal_override?: StateOverride,
  adv_override?: AdvStateOverride,
  juice_info = useRuntimeStore().histogram.result?.juice_info ?? null,
): OneUpgradeInput[] {
  const upgrade_map = get_upgrade_map(upgrade_arr, tier);
  return Object.entries(keyed_upgrades)
    .filter((x) => tier === 0 || x[1].is_normal_honing) // shouldn't really be necessary but apparently it went  wrong once somehow so adding this guard here
    .filter((x) => tier === 1 || x[1].piece_index < NUM_ADV_PIECES)
    .map(([key, one_upgrade_input]) => {
      const upgrade = upgrade_map.get(key) ?? null;

      one_upgrade_input.double_balls =
        !one_upgrade_input.is_normal_honing &&
        (ADV_ALWAYS_DOUBLE_BALLS ||
          (one_upgrade_input.upgrade_index < 2 && express) ||
          (one_upgrade_input.upgrade_index >= 2 && pretend_30_40));
      // console.log(one_upgrade_input.double_balls);
      let out = structuredClone(toRaw(one_upgrade_input));
      if (
        !(upgrade && upgrade.state && upgrade.state.length > 0 && juice_info)
      ) {
        return out;
      }

      let relevant_id_map = upgrade.is_normal_honing
        ? juice_info.normal_uindex_to_id
        : juice_info.adv_uindex_to_id;

      let relevant_upgrade =
        relevant_id_map[upgrade.piece_type_usize][upgrade.upgrade_index];
      // console.log(adv_override);
      out.unlocked = out.is_normal_honing
        ? out.starting_artisan > 0 || out.starting_num_taps > 0
        : out.adv_progress !== null &&
          (out.adv_progress[0] > 0 || out.adv_progress[1] > 0);

      if (out.state !== null && out.state.length === 0) {
        // special cased, reset will wipe it like this, so dont copy from optimizer bundle
        // console.log("overwritten");
        one_upgrade_input.state = null; // so the next time it doesn't overwrite
        return out;
      }
      out.state = upgrade.is_normal_honing
        ? upgrade.state.slice(out.taps_since_last_input).map((one_state) =>
            normal_override === undefined
              ? one_state
              : one_state
                  .filter((id) =>
                    normal_override.juice == NormalOverride.Empty && id <= 1
                      ? false
                      : true,
                  )
                  .concat(
                    relevant_upgrade.filter(
                      (id) =>
                        normal_override.juice == NormalOverride.Full && id <= 1,
                    ),
                  )
                  .filter((id) =>
                    normal_override.book == NormalOverride.Empty && id > 1
                      ? false
                      : true,
                  )
                  .concat(
                    relevant_upgrade.filter(
                      (id) =>
                        normal_override.book == NormalOverride.Full && id > 1,
                    ),
                  ),
          )
        : upgrade.state.map((one_state, index) => [
            adv_override === undefined ||
            (index == 0 ? adv_override.juice : adv_override.scroll) ==
              AdvOverride.Optimizer
              ? one_state[0]
              : (index == 0 ? adv_override.juice : adv_override.scroll) ==
                  AdvOverride.Empty
                ? 0
                : (index == 0 ? adv_override.juice : adv_override.scroll) ==
                    AdvOverride.Grace
                  ? GRACE_FIRST_N.length - 1
                  : JOINED_ADV_JUICE.length - 1,
          ]);

      return out;
    });
}

export function special_sort_override(
  special_state: number[],
  upgrade_arr: Upgrade[],
  special_override?: SpecialOverride,
): number[] {
  // console.log(
  //   special_state,
  //   special_override,
  //   special_override === undefined,
  //   special_state === undefined,
  //   special_override?.optimizer,
  // );
  if (
    special_override === undefined ||
    special_state === undefined ||
    special_override.optimizer
  ) {
    return special_state;
  }

  let out = structuredClone(special_state);
  if (special_override.highest_first) {
    // For each piece_index, find the max upgrade_index among the indices present
    // in special_state. Upgrades that are the peak of their piece_index float
    // to the front; all others sink to the back. weapon_first does not apply.
    const peak_per_piece = new Map<number, number>();
    for (const idx of special_state) {
      const u = upgrade_arr[idx];
      const current = peak_per_piece.get(u.piece_index);
      if (current === undefined || u.upgrade_index > current) {
        peak_per_piece.set(u.piece_index, u.upgrade_index);
      }
    }

    const is_peak = (idx: number): boolean => {
      const u = upgrade_arr[idx];
      return peak_per_piece.get(u.piece_index) === u.upgrade_index;
    };

    out.sort((a, b) => {
      const a_peak = is_peak(a);
      const b_peak = is_peak(b);
      const ua = upgrade_arr[a];
      const ub = upgrade_arr[b];

      const a_preferred =
        (ua.piece_type_usize === 1) === special_override.weapon_first;
      const b_preferred =
        (ub.piece_type_usize === 1) === special_override.weapon_first;
      if (ua.is_normal_honing !== ub.is_normal_honing) {
        return ua.is_normal_honing ? -1 : 1;
      } else if (a_peak !== b_peak) {
        return a_peak ? -1 : 1;
      } else if (a_preferred !== b_preferred) {
        return a_preferred ? -1 : 1;
      }
      return 0;
    });
  } else {
    // Sort ascending by upgrade_index, with weapon_first as the primary key.
    out.sort((a, b) => {
      const ua = upgrade_arr[a];
      const ub = upgrade_arr[b];
      const a_preferred =
        (ua.piece_type_usize === 1) === special_override.weapon_first;
      const b_preferred =
        (ub.piece_type_usize === 1) === special_override.weapon_first;
      if (ua.is_normal_honing !== ub.is_normal_honing) {
        return ua.is_normal_honing ? -1 : 1;
      } else if (a_preferred !== b_preferred) {
        return a_preferred ? -1 : 1;
      } else {
        return ua.upgrade_index - ub.upgrade_index;
      }
    });
  }
  // console.log("sorted", out);
  return out;
}
function apply_tax(x: number): number {
  return Math.max(Math.min(1, x), Math.floor(x * 0.95));
}

// A lower tier's stock counted as the tier's materials it converts to, which is how Rust sees a
// Serca character's T4: bound into bound, roster-bound and tradable into roster-bound (converting
// binds it). Floored per band, like the market page's convert button. A disabled material on
// either side is left out.
export function merge_lower_tier(
  tier: number,
  bound: Record<string, number>,
  roster: Record<string, number>,
  tradable: Record<string, number>,
  enabled: Record<string, boolean> = {},
) {
  const out = {
    bound: { ...bound },
    roster: { ...roster },
    tradable: { ...tradable },
  };
  for (const mat of CONVERTIBLE_MATERIALS) {
    const { label, ratio } = mat.from;
    if (!mat.tiers.includes(tier)) continue;
    if (enabled[mat.label] === false || enabled[label] === false) continue;
    const units = (amount: number) => Math.floor(amount / ratio);
    out.bound[mat.label] = (bound[mat.label] ?? 0) + units(bound[label] ?? 0);
    out.roster[mat.label] =
      (roster[mat.label] ?? 0) +
      units((roster[label] ?? 0) + (tradable[label] ?? 0));
  }
  return out;
}

export function build_materials(): Record<string, OneMaterial> {
  const roster_store = useRosterStore();
  const {
    active_profile,
    active_roster_mats_owned,
    active_tradable_mats_owned,
  } = storeToRefs(roster_store);
  // Prices here are per unit: which bundle a material is sold in, and whether it is
  // cheaper to convert one up from a lower tier, are both resolved by the store.
  return materials_of(
    active_profile.value.tier,
    input_column_to_num(active_profile.value.bound_budgets),
    input_column_to_num(active_roster_mats_owned.value),
    input_column_to_num(active_tradable_mats_owned.value),
    active_profile.value.bound_budgets.enabled,
    roster_store.active_unit_prices,
    roster_store.active_effective_prices,
  );
}

function materials_of(
  tier: number,
  bound: Record<string, number>,
  roster_owned: Record<string, number>,
  tradable_owned: Record<string, number>,
  enabled: Record<string, boolean>,
  listed: Record<string, number>,
  effective: Record<string, number>,
): Record<string, OneMaterial> {
  const merged = merge_lower_tier(
    tier,
    bound,
    roster_owned,
    tradable_owned,
    enabled,
  );
  const out: Record<string, OneMaterial> = {};
  for (const label of ALL_LABELS[tier]) {
    // A disabled material is one the user will not trade, which `bound` already models
    // as an effectively infinite stock. Zero prices keep it out of the gold entirely, so
    // what its leftovers are "worth" cannot depend on the plan.
    const traded = enabled[label];
    out[label] = {
      bound: merged.bound[label] ?? 0,
      roster: merged.roster[label] ?? 0,
      // gold is never sold
      tradable: !traded || label === "Gold" ? 0 : (merged.tradable[label] ?? 0),
      taxed_price: traded ? apply_tax(listed[label]) : 0,
      market_price: traded ? effective[label] : 0,
    };
  }
  return out;
}

export function build_payload(override?: OptimizerOverride): Payload {
  const runtime = useRuntimeStore();
  const { active_profile, processed_chests } = storeToRefs(useRosterStore());
  const tier = active_profile.value.tier;
  // console.log(runtime.optimizer.result?.adv_cache);
  return {
    material_labels: ALL_LABELS[tier],
    materials: build_materials(),
    // Rust asserts the plan does not decrease, and the dropdowns only disable the values
    // that would break that rather than preventing them. Clamping here also copies the
    // plan out of the reactive proxy, which postMessage cannot clone.
    plans: [ALL_WORTHLESS, clamp_band_plan(active_profile.value.band_values)],
    optimizer_plan: 1,
    upgrade_info: keyed_to_array(
      active_profile.value.keyed_upgrades,
      runtime.optimizer.result?.upgrade_arr,
      tier,
      active_profile.value.express_event,
      active_profile.value.pretend_30_40_x2_grace,
      override?.normal,
      override?.advanced,
    ),
    // a Serca character's T4 special leaps count too
    special_budget: merge_lower_tier(
      tier,
      input_column_to_num(active_profile.value.special_budget),
      {},
      {},
    ).bound[SPECIAL_LEAP_LABELS[tier]],
    express_event: false, //active_profile.value.express_event, temporary overwrite while event doesn't do anything
    tier,
    min_resolution: active_profile.value.min_resolution,
    num_threads: 1,
    metric_type: 1,
    special_state: special_sort_override(
      toRaw(runtime.optimizer.result?.special_state),
      runtime.optimizer.result?.upgrade_arr,
      override?.special,
    ),
    // Amounts stay fractional and Rust floors the total.
    chests: Object.fromEntries(
      processed_chests.value.map((chest) => [
        chest.key,
        { count: chest.count, band: chest.band, options: chest.options },
      ]),
    ),
    chest_states: toRaw(runtime.optimizer.result?.chest_states),
    special_label: SPECIAL_LEAP_LABELS[tier],
    adv_cache: toRaw(runtime.adv_cache),
  };
}

// The payload of a profile's current upgrades and settings over given budgets, chests and prices,
// with no optimizer state. Plain, so it can be hashed and posted.
export function payload_for(
  profile: CharProfile,
  bound: Record<string, number>,
  roster: Record<string, number>,
  tradable: Record<string, number>,
  special: Record<string, number>,
  chests: Chest[],
  prices: Record<string, number>,
  bundles: Record<string, number>,
): Payload {
  const tier = profile.tier;
  const listed = unit_prices(
    {
      ...Object.fromEntries(ALL_BUNDLE_KEYS.map((key) => [key, 0])),
      ...prices,
    },
    bundles ?? {},
  );
  const upgrade_info = keyed_to_array(
    profile.keyed_upgrades,
    null,
    tier,
    profile.express_event,
    profile.pretend_30_40_x2_grace,
  ).map(
    ({
      expanded: _expanded,
      used_materials: _used,
      taps_since_last_input: _taps,
      ...rest
    }) => rest,
  );
  return JSON.parse(
    JSON.stringify({
      material_labels: ALL_LABELS[tier],
      materials: materials_of(
        tier,
        bound,
        roster,
        tradable,
        profile.bound_budgets.enabled,
        listed,
        effective_unit_prices(listed),
      ),
      plans: [ALL_WORTHLESS, clamp_band_plan(profile.band_values)],
      optimizer_plan: 1,
      upgrade_info,
      special_budget:
        merge_lower_tier(tier, special, {}, {}).bound[
          SPECIAL_LEAP_LABELS[tier]
        ] ?? 0,
      express_event: false,
      tier,
      min_resolution: profile.min_resolution,
      num_threads: 1,
      metric_type: 1,
      chests: Object.fromEntries(
        chests.map((chest) => [
          chest.key,
          { count: chest.count, band: chest.band, options: chest.options },
        ]),
      ),
      special_label: SPECIAL_LEAP_LABELS[tier],
      adv_cache: null,
    }),
  );
}

// the histogram's payload of what an optimizer run of `payload` chose
export function states_payload(
  payload: Payload,
  profile: CharProfile,
  state: any,
): Payload {
  return {
    ...payload,
    upgrade_info: keyed_to_array(
      profile.keyed_upgrades,
      state.upgrade_arr,
      profile.tier,
      profile.express_event,
      profile.pretend_30_40_x2_grace,
      undefined,
      undefined,
      state.prep_output.juice_info,
    ),
    special_state: state.special_state,
    chest_states: state.chest_states,
    adv_cache: state.adv_cache,
  };
}
