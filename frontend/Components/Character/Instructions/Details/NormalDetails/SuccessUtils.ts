import { useRosterStore } from "@/Stores/RosterConfig";
import { ALL_LABELS, by_label } from "@/Utils/Constants";
import { input_column_to_num, InputColumn } from "@/Utils/InputColumn";
import { Upgrade, UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { toRaw } from "vue";
import { grid_change_callback } from "@/Components/Character/CharWorkerUtils";
import { useRuntimeStore } from "@/Stores/RuntimeState";

export interface BudgetSnapshot {
  bound_budgets: InputColumn;
  roster_mats: InputColumn;
  tradable_mats: InputColumn;
}
// keyed by material label, like everything on this side of the wasm boundary
export interface RemainingMats {
  bound_budgets: Record<string, number>;
  roster_mats: Record<string, number>;
  tradable_mats: Record<string, number>;
}

export function mark_upgrade_as_done(upgrade: Upgrade) {
  const { active_profile } = storeToRefs(useRosterStore());
  if (upgrade.is_normal_honing) {
    active_profile.value.normal_grid[upgrade.piece_index][
      upgrade.upgrade_index
    ] = UpgradeStatus.Done;
  } else {
    active_profile.value.adv_grid[upgrade.piece_index][upgrade.upgrade_index] =
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

  // console.log(
  //   pretend_zero_no_unlock,
  //   upgrade.starting_num_taps,
  //   taps_since_last_run,
  // );
  for (let cost_type = 0; cost_type < 7; cost_type++) {
    out[cost_type] =
      upgrade.unlock_costs[cost_type] *
        (pretend_zero_no_unlock && taps_since_last_run === 0
          ? 0
          : upgrade.starting_num_taps !== 0
            ? 0
            : 1) +
      upgrade.costs[cost_type] * taps_since_last_run;
  }

  let relevant_id_map = upgrade.is_normal_honing
    ? juice_info.normal_uindex_to_id
    : juice_info.adv_uindex_to_id;
  // console.log(relevant_id_map[upgrade.upgrade_index])
  for (const id of relevant_id_map[upgrade.piece_type_usize][
    upgrade.upgrade_index
  ]) {
    let juice_cost = 0;

    let juice_type = juice_info.all_juices[id][upgrade.piece_type_usize].get(
      upgrade.upgrade_index,
    );
    // console.log(juice_info.all_juices);
    let amt = upgrade.is_normal_honing
      ? juice_type.normal_amt_used
      : juice_type.adv_amt_used;

    if (upgrade.is_normal_honing) {
      for (
        let index = 0;
        index < Math.min(taps_since_last_run, upgrade.normal_dist.length - 2);
        index++
      ) {
        if (upgrade.state[index].includes(id)) {
          juice_cost += amt;
        }
        // console.log(
        //   // juice_cost,
        //   upgrade.state[index],
        //   id,
        //   upgrade.state[index],
        // );
      }
    } else {
      if (id <= 1) {
        juice_cost = adv_juice_used * amt;
      } else {
        juice_cost = adv_scroll_used * amt;
      }
    }
    console.log(
      juice_cost,
      upgrade.state,
      relevant_id_map[upgrade.piece_type_usize][upgrade.upgrade_index],
      id,
      taps_since_last_run,
      upgrade.normal_dist.length,
    );
    out[7 + id] = juice_cost;
  }
  console.log(out);
  return out;
}
export function make_budget_snapshot(): BudgetSnapshot {
  const {
    active_profile,
    active_roster_mats_owned,
    active_tradable_mats_owned,
  } = storeToRefs(useRosterStore());
  // console.log("snap");
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
  const { active_profile } = storeToRefs(useRosterStore());
  const tier = active_profile.value.tier;
  const previous_budgets: BudgetSnapshot =
    inp_previous_budget ?? make_budget_snapshot();

  const bound_owned = input_column_to_num(previous_budgets.bound_budgets);
  const roster_owned = input_column_to_num(previous_budgets.roster_mats);
  const tradable_owned = input_column_to_num(previous_budgets.tradable_mats);
  // Rust reports costs by material row; this is where they become labels
  const cost = by_label(used_materials, tier);

  const bound_budgets: Record<string, number> = {};
  const roster_mats: Record<string, number> = {};
  const tradable_mats: Record<string, number> = {};
  for (const label of ALL_LABELS[tier]) {
    // spend bound first, then roster-bound, then tradable
    let remaining_cost = Math.max(0, cost[label] ?? 0);
    const spend = (owned: number) => {
      const deduct = Math.min(owned, remaining_cost);
      remaining_cost -= deduct;
      return Math.max(0, owned - deduct);
    };
    bound_budgets[label] = spend(bound_owned[label]);
    roster_mats[label] = spend(roster_owned[label]);
    tradable_mats[label] = spend(tradable_owned[label]);
  }
  return { bound_budgets, roster_mats, tradable_mats };
}

export function apply_remaining_mats() {
  const runtime = useRuntimeStore();
  const {
    active_profile,
    roster_config,
    active_roster_mats_owned,
    active_tradable_mats_owned,
  } = storeToRefs(useRosterStore());

  if (!roster_config.value.auto_deduct_costs) {
    return;
  }
  const tier = active_profile.value.tier;
  if (runtime.budget_snapshot === null) {
    runtime.budget_snapshot = make_budget_snapshot();
  }

  // used_materials comes back from Rust indexed by material row, so it is summed as rows
  const total_used = Object.values(active_profile.value.keyed_upgrades)
    .map((u) => u.used_materials)
    .reduce(
      (acc, cur) => acc.map((x, i) => x + (cur?.[i] ?? 0)),
      Array(ALL_LABELS[tier].length).fill(0),
    );
  const remaining: RemainingMats = compute_remaininig_materials(
    total_used,
    runtime.budget_snapshot,
  );

  const write = (column: InputColumn, values: Record<string, number>) => {
    for (const label of ALL_LABELS[tier]) {
      if (column.enabled[label]) {
        column.values[label] = values[label].toLocaleString();
      }
    }
  };
  write(active_profile.value.bound_budgets, remaining.bound_budgets);
  write(active_roster_mats_owned.value, remaining.roster_mats);
  write(active_tradable_mats_owned.value, remaining.tradable_mats);
}
