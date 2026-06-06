import {
  ALL_LABELS,
  BUNDLE_SIZE,
  GRACE_FIRST_N,
  JOINED_ADV_JUICE,
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
  normal_override?: StateOverride,
  adv_override?: AdvStateOverride,
): OneUpgradeInput[] {
  const { active_profile } = storeToRefs(useRosterStore());

  const juice_info =
    active_profile.value.histogram_worker_bundle.result?.juice_info ?? null;
  const upgrade_map = get_upgrade_map(upgrade_arr, tier);
  return Object.entries(keyed_upgrades).map(([key, one_upgrade_input]) => {
    const upgrade = upgrade_map.get(key) ?? null;
    let out = structuredClone(toRaw(one_upgrade_input));
    if (!(upgrade && upgrade.state && upgrade.state.length > 0 && juice_info)) {
      return out;
    }

    let relevant_id_map = upgrade.is_normal_honing
      ? juice_info.normal_uindex_to_id
      : juice_info.adv_uindex_to_id;

    let relevant_upgrade = relevant_id_map[upgrade.upgrade_index];
    // console.log(adv_override);
    out.unlocked = out.is_normal_honing
      ? out.starting_artisan > 0 || out.starting_num_taps > 0
      : out.adv_progress !== null &&
        (out.adv_progress[0] > 0 || out.adv_progress[1] > 0);

    if (out.state !== null && out.state.length === 0) {
      // special cased, reset will wipe it like this, so dont copy from optimizer bundle
      return out;
    }
    out.state = upgrade.state
      .slice(out.taps_since_last_input)
      .map((x, index) =>
        upgrade.is_normal_honing
          ? [
              normal_override === undefined ||
              normal_override.juice == NormalOverride.Optimizer
                ? x[0]
                : normal_override.juice == NormalOverride.Empty
                  ? false
                  : true,
              normal_override === undefined ||
              normal_override.book == NormalOverride.Optimizer
                ? x[1]
                : normal_override.book == NormalOverride.Empty
                  ? 0
                  : relevant_upgrade[relevant_upgrade.length - 1],
            ]
          : [
              false,
              adv_override === undefined ||
              (index == 0 ? adv_override.juice : adv_override.scroll) ==
                AdvOverride.Optimizer
                ? x[1]
                : (index == 0 ? adv_override.juice : adv_override.scroll) ==
                    AdvOverride.Empty
                  ? 0
                  : (index == 0 ? adv_override.juice : adv_override.scroll) ==
                      AdvOverride.Grace
                    ? GRACE_FIRST_N.length - 1
                    : JOINED_ADV_JUICE.length - 1,
            ],
      );
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
    // For each piece_type, find the max upgrade_index among the indices present
    // in special_state. Upgrades that are the peak of their piece_type float
    // to the front; all others sink to the back. weapon_first does not apply.
    const peak_per_piece = new Map<number, number>();
    for (const idx of special_state) {
      const u = upgrade_arr[idx];
      const current = peak_per_piece.get(u.piece_type);
      if (current === undefined || u.upgrade_index > current) {
        peak_per_piece.set(u.piece_type, u.upgrade_index);
      }
    }

    const is_peak = (idx: number): boolean => {
      const u = upgrade_arr[idx];
      return peak_per_piece.get(u.piece_type) === u.upgrade_index;
    };

    out.sort((a, b) => {
      const a_peak = is_peak(a);
      const b_peak = is_peak(b);
      const ua = upgrade_arr[a];
      const ub = upgrade_arr[b];

      const a_preferred = ua.is_weapon === special_override.weapon_first;
      const b_preferred = ub.is_weapon === special_override.weapon_first;
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
      const a_preferred = ua.is_weapon === special_override.weapon_first;
      const b_preferred = ub.is_weapon === special_override.weapon_first;
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

export function build_material_info(): OneMaterialInput[] {
  const roster_store = useRosterStore();
  const { active_profile } = storeToRefs(roster_store);
  const {
    roster_config,
    active_roster_mats_owned,
    active_tradable_mats_owned,
    active_region,
  } = storeToRefs(useRosterStore());

  const tier = active_profile.value.tier;
  const bound_budgets = input_column_to_num(
    active_profile.value.bound_budgets[tier],
  );
  const enabled = active_profile.value.bound_budgets[tier].enabled;
  const roster_mats_owned = input_column_to_num(
    active_roster_mats_owned.value[tier],
  );
  const tradable_mats_owned = input_column_to_num(
    active_tradable_mats_owned.value[tier],
  );

  const leftover_price = input_column_to_num(
    active_profile.value.leftover_price[tier],
  );
  const effective_price =
    tier == 0
      ? input_column_to_num(roster_store.active_mats_prices[tier])
      : roster_config.value.effective_serca_price;

  const tradable_mats_price = input_column_to_num(
    roster_store.active_mats_prices[tier],
  ).map(
    (x: number, index: number) =>
      Math.max(Math.min(1, x), Math.floor(x * 0.95)) /
      (ALL_LABELS[active_profile.value.tier][index] == "Shards"
        ? roster_config.value.selected_shard_bag_size[active_region.value]
        : BUNDLE_SIZE[index]),
  );
  const mats_prices = effective_price.map(
    (x: number, index: number) =>
      x /
      (ALL_LABELS[active_profile.value.tier][index] == "Shards"
        ? roster_config.value.selected_shard_bag_size[active_region.value]
        : BUNDLE_SIZE[index]),
  );
  // console.log()
  return ALL_LABELS[tier].map((_, index) => [
    [0, 0],
    [bound_budgets[index], leftover_price[index]],
    [roster_mats_owned[index], tradable_mats_price[index]],
    [
      !enabled[index] || index == 5 ? 0 : tradable_mats_owned[index],
      mats_prices[index],
    ], // disabled mats shouldn't be sold either, disregard tradable gold
  ]);
}

export function build_payload(override?: OptimizerOverride): Payload {
  const { active_profile } = storeToRefs(useRosterStore());
  const tier = active_profile.value.tier;
  // console.log(override);
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
      active_profile.value.optimizer_worker_bundle.result?.upgrade_arr,
      tier,
      override?.normal,
      override?.advanced,
    ),
    special_budget: input_column_to_num(active_profile.value.special_budget)[0],
    express_event: active_profile.value.express_event,
    tier,
    min_resolution: active_profile.value.min_resolution,
    num_threads: 1,
    metric_type: 1,
    special_state: special_sort_override(
      toRaw(active_profile.value.optimizer_worker_bundle.result?.special_state),
      active_profile.value.optimizer_worker_bundle.result?.upgrade_arr,
      override?.special,
    ),
  };
}
