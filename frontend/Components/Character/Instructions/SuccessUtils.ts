import { useRosterStore } from "@/Stores/RosterConfig";
import { input_column_to_num, InputColumn } from "@/Utils/InputColumn";
import { to_upgrade_key, Upgrade, UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { toRaw } from "vue";
import { grid_change_callback } from "../CharWorkerUtils";

export interface BudgetSnapshot {
  bound_budgets: InputColumn[];
  roster_mats: InputColumn[];
  tradable_mats: InputColumn[];
}
export interface RemainingMats {
  bound_budgets: number[];
  roster_mats: number[];
  tradable_mats: number[];
}

export function mark_upgrade_as_done(upgrade: Upgrade) {
  const { active_profile } = storeToRefs(useRosterStore());
  if (upgrade.is_normal_honing) {
    active_profile.value.normal_grid[upgrade.piece_type][
      upgrade.upgrade_index
    ] = UpgradeStatus.Done;
  } else {
    active_profile.value.adv_grid[upgrade.piece_type][upgrade.upgrade_index] =
      UpgradeStatus.Done;
  }
  grid_change_callback();
}
export function compute_used_materials(
  upgrade: Upgrade,
  taps_since_last_run: number,
  juice_info: any,
  adv_juice_used: number,
  adv_scroll_used: number,
  pretend_zero_no_unlock: boolean,
): number[] {
  if (!upgrade.cost_dist) return [];
  let out = new Array(upgrade.cost_dist.length).fill(0);

  for (let cost_type = 0; cost_type < 7; cost_type++) {
    out[cost_type] =
      upgrade.unlock_costs[cost_type] *
        (pretend_zero_no_unlock && taps_since_last_run == 0 ? 0 : 1) +
      upgrade.costs[cost_type] * taps_since_last_run;
  }

  let relevant_id_map = upgrade.is_normal_honing
    ? juice_info.normal_uindex_to_id
    : juice_info.adv_uindex_to_id;
  // console.log(relevant_id_map[upgrade.upgrade_index])
  for (const id of relevant_id_map[upgrade.upgrade_index]) {
    let juice_cost = 0;

    let juice_type = juice_info.all_juices[id].data.get(
      String(upgrade.upgrade_index),
    );
    let amt = upgrade.is_normal_honing
      ? juice_type.normal_amt_used
      : juice_type.adv_amt_used;

    if (upgrade.is_normal_honing) {
      for (
        let index = 0;
        index < Math.min(taps_since_last_run, upgrade.normal_dist.length - 2);
        index++
      ) {
        if (
          (upgrade.state[index][0] === true && id == 0) ||
          (upgrade.state[index][1] === id && id !== 0)
        ) {
          juice_cost += amt;
        }
        // console.log(juice_cost)
      }
    } else {
      if (id === 0) {
        juice_cost = adv_juice_used * amt;
      } else {
        juice_cost = adv_scroll_used * amt;
      }
    }

    out[7 + id + (upgrade.is_weapon ? 0 : juice_info.num_juice_avail)] =
      juice_cost;
  }
  return out;
}
export function make_budget_snapshot(): BudgetSnapshot {
  const {
    active_profile,
    active_roster_mats_owned,
    active_tradable_mats_owned,
  } = storeToRefs(useRosterStore());
  console.log("snap");
  return {
    bound_budgets: structuredClone(toRaw(active_profile.value.bound_budgets)),
    roster_mats: structuredClone(toRaw(active_roster_mats_owned.value)),
    tradable_mats: structuredClone(toRaw(active_tradable_mats_owned.value)),
  };
}
export function compute_remaininig_materials(
  used_materials: number[],
  inp_previous_budget?: BudgetSnapshot,
): RemainingMats {
  console.log(used_materials, inp_previous_budget);
  const { active_profile } = storeToRefs(useRosterStore());
  const tier = active_profile.value.tier;
  const previous_budgets: BudgetSnapshot = inp_previous_budget
    ? inp_previous_budget
    : make_budget_snapshot();
  const bound_budgets: number[] = [];
  const roster_mats: number[] = [];
  const tradable_mats: number[] = [];
  used_materials.forEach((cost, index) => {
    if (cost <= 0) {
      bound_budgets.push(
        input_column_to_num(previous_budgets.bound_budgets[tier])[index],
      );
      roster_mats.push(
        input_column_to_num(previous_budgets.roster_mats[tier])[index],
      );
      tradable_mats.push(
        input_column_to_num(previous_budgets.tradable_mats[tier])[index],
      );
      return;
    }
    let remaining_cost = cost;
    // 1. Bound
    let bound_owned = input_column_to_num(previous_budgets.bound_budgets[tier])[
      index
    ];
    let deduct_bound = Math.min(bound_owned, remaining_cost);
    bound_budgets.push(Math.max(0, bound_owned - deduct_bound));
    remaining_cost -= deduct_bound;
    // 2. Roster
    let roster_owned = input_column_to_num(previous_budgets.roster_mats[tier])[
      index
    ];
    if (
      remaining_cost > 0 &&
      previous_budgets.roster_mats[index] !== undefined
    ) {
      let deduct_roster = Math.min(roster_owned, remaining_cost);
      roster_mats.push(Math.max(0, roster_owned - deduct_roster));
      remaining_cost -= deduct_roster;
    } else {
      roster_mats.push(roster_owned);
    }
    // 3. Tradable
    let tradable_owned = input_column_to_num(
      previous_budgets.tradable_mats[tier],
    )[index];
    if (
      remaining_cost > 0 &&
      previous_budgets.tradable_mats[index] !== undefined
    ) {
      let deduct_tradable = Math.min(tradable_owned, remaining_cost);
      tradable_mats.push(Math.max(0, tradable_owned - deduct_tradable));
    } else {
      tradable_mats.push(tradable_owned);
    }
  });
  // console.log("computed")
  return { bound_budgets, roster_mats, tradable_mats };
}

export function apply_remaining_mats(upgrade: Upgrade) {
  const {
    active_profile,
    roster_config,
    active_roster_mats_owned,
    active_tradable_mats_owned,
  } = storeToRefs(useRosterStore());
  const tier = active_profile.value.tier;
  const this_keyed =
    active_profile.value.keyed_upgrades[
      to_upgrade_key(
        upgrade.piece_type,
        upgrade.upgrade_index,
        upgrade.is_normal_honing,
        active_profile.value.tier,
      )
    ];
  if (roster_config.value.budget_snapshot === null) {
    // console.log(roster_config.value.budget_snapshot);
    roster_config.value.budget_snapshot = make_budget_snapshot();
    // console.log(
    //   "snap",
    //   toRaw(roster_config.value.budget_snapshot.bound_budgets[0].data),
    // );
    // console.log(roster_config.value.budget_snapshot);
  }
  // console.log(
  //   "calc",
  //   upgrade.starting_num_taps,
  //   taps_since_last_input.value,
  //   toRaw(roster_config.value.budget_snapshot.bound_budgets[0].data),
  // );
  const remaining_materials: RemainingMats = compute_remaininig_materials(
    Object.values(active_profile.value.keyed_upgrades)
      .map((u) => u.used_materials)
      .reduce(
        (acc, cur) => acc.map((x, i) => x + (cur?.[i] ?? 0)),
        Array(this_keyed.used_materials.length).fill(0),
      ),
    roster_config.value.budget_snapshot,
  );

  // console.log(remaining_materials.bound_budgets);

  this_keyed.used_materials.forEach((_, index) => {
    if (active_profile.value.bound_budgets[tier].enabled[index]) {
      active_profile.value.bound_budgets[tier].data[index] =
        remaining_materials.bound_budgets[index].toLocaleString();
    }
    if (active_roster_mats_owned.value[tier].enabled[index]) {
      active_roster_mats_owned.value[tier].data[index] =
        remaining_materials.roster_mats[index].toLocaleString();
    }
    if (active_tradable_mats_owned.value[tier].enabled[index]) {
      active_tradable_mats_owned.value[tier].data[index] =
        remaining_materials.tradable_mats[index].toLocaleString();
    }
  });
}
