import {
  GRACE_FIRST_N,
  Material,
  JOINED_ADV_JUICE,
  NUM_ADV_PIECES,
  SPECIAL_LEAP_LABELS,
  TIER_MATERIALS,
} from "@/Utils/Constants";
import { TreatmentPlan } from "@/Stores/CharacterProfile";

import { toRaw } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  get_upgrade_map,
  KeyedUpgrades,
  OneMaterialInput,
  OneUpgradeInput,
  Upgrade,
} from "@/Utils/KeyedUpgrades";
import { input_column_to_num } from "@/Utils/InputColumn";
import { storeToRefs } from "pinia";
import { useRuntimeStore } from "@/Stores/RuntimeState";

// I don't think it's possible to directly export this struct from rust to javascript because of all the vectors,
// so it's copied & pasted here
export interface Payload {
  material_info: number[][][];
  optimizer_plan?: number[];
  upgrade_info: OneUpgradeInput[];
  special_budget: number;
  special_state?: number[];
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
): OneUpgradeInput[] {
  const juice_info = useRuntimeStore().histogram.result?.juice_info ?? null;
  const upgrade_map = get_upgrade_map(upgrade_arr, tier);
  return Object.entries(keyed_upgrades)
    .filter((x) => tier === 0 || x[1].is_normal_honing) // shouldn't really be necessary but apparently it went  wrong once somehow so adding this guard here
    .filter((x) => tier === 1 || x[1].piece_index < NUM_ADV_PIECES)
    .map(([key, one_upgrade_input]) => {
      const upgrade = upgrade_map.get(key) ?? null;

      one_upgrade_input.double_balls =
        upgrade !== null &&
        !upgrade.is_normal_honing &&
        ((upgrade.upgrade_index < 2 && express) ||
          (upgrade.upgrade_index >= 2 && pretend_30_40));
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

export function build_material_info(): OneMaterialInput[] {
  const roster_store = useRosterStore();
  const {
    active_profile,
    active_roster_mats_owned,
    active_tradable_mats_owned,
  } = storeToRefs(roster_store);

  const tier = active_profile.value.tier;
  const bound = input_column_to_num(active_profile.value.bound_budgets);
  const enabled = active_profile.value.bound_budgets.enabled;
  const roster_owned = input_column_to_num(active_roster_mats_owned.value);
  const tradable_owned = input_column_to_num(active_tradable_mats_owned.value);
  const leftover = input_column_to_num(active_profile.value.leftover_price);

  // Prices here are per unit: which bundle a material is sold in, and whether it is
  // cheaper to convert one up from a lower tier, are both resolved by the store.
  const listed = roster_store.active_unit_prices;
  const effective = roster_store.active_effective_prices;

  return TIER_MATERIALS[tier].map((mat) => [
    [0, 0],
    [bound[mat.label], leftover[mat.label]],
    [roster_owned[mat.label], apply_tax(listed[mat.label])],
    [
      // disabled mats shouldn't be sold either, and gold is never sold
      !enabled[mat.label] || mat.label === "Gold"
        ? 0
        : tradable_owned[mat.label],
      effective[mat.label],
    ],
  ]);
}

export function build_payload(override?: OptimizerOverride): Payload {
  const runtime = useRuntimeStore();
  const { active_profile } = storeToRefs(useRosterStore());
  const tier = active_profile.value.tier;
  // console.log(runtime.optimizer.result?.adv_cache);
  return {
    material_info: build_material_info(),
    optimizer_plan:
      // wasm_op == WasmOp.OptimizeAverage
      active_profile.value.optimizer_treatment_plan ===
      TreatmentPlan.TreatRosterAsBound
        ? [0, 0, 2, 3]
        : active_profile.value.optimizer_treatment_plan ===
            TreatmentPlan.TreatTradableAsBound
          ? [0, 0, 0, 3]
          : [0, 1, 2, 3], //this  shouldn't happen
    // : null,
    upgrade_info: keyed_to_array(
      active_profile.value.keyed_upgrades,
      runtime.optimizer.result?.upgrade_arr,
      tier,
      active_profile.value.express_event,
      active_profile.value.pretend_30_40_x2_grace,
      override?.normal,
      override?.advanced,
    ),
    special_budget: input_column_to_num(active_profile.value.special_budget)[
      SPECIAL_LEAP_LABELS[active_profile.value.tier]
    ],
    express_event: active_profile.value.express_event,
    tier,
    min_resolution: active_profile.value.min_resolution,
    num_threads: 1,
    metric_type: 1,
    special_state: special_sort_override(
      toRaw(runtime.optimizer.result?.special_state),
      runtime.optimizer.result?.upgrade_arr,
      override?.special,
    ),
    adv_cache: toRaw(runtime.adv_cache),
  };
}
