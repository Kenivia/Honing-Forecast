import { useRosterStore } from "@/Stores/RosterConfig";
import {
  ALL_LABELS,
  MATERIALS,
  NUM_BASE_MATS,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import { merge_lower_tier } from "@/WasmInterface/PayloadBuilder";
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
  tier: number,
): Record<string, number> {
  if (!upgrade.cost_dist) return {};
  const labels = ALL_LABELS[tier];
  const out: Record<string, number> = Object.fromEntries(
    labels.map((label) => [label, 0]),
  );

  for (let cost_type = 0; cost_type < NUM_BASE_MATS; cost_type++) {
    out[labels[cost_type]] =
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
    out[labels[NUM_BASE_MATS + id]] = juice_cost;
  }
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
  used_materials: Record<string, number>,
  inp_previous_budget?: BudgetSnapshot,
): RemainingMats {
  const { active_profile } = storeToRefs(useRosterStore());
  const tier = active_profile.value.tier;
  const previous_budgets: BudgetSnapshot =
    inp_previous_budget ?? make_budget_snapshot();

  const bound_owned = input_column_to_num(previous_budgets.bound_budgets);
  const roster_owned = input_column_to_num(previous_budgets.roster_mats);
  const tradable_owned = input_column_to_num(previous_budgets.tradable_mats);
  const enabled = previous_budgets.bound_budgets.enabled;
  const cost = used_materials;

  const bound_budgets: Record<string, number> = {};
  const roster_mats: Record<string, number> = {};
  const tradable_mats: Record<string, number> = {};
  for (const label of ALL_LABELS[tier]) {
    // the lower-tier source Rust counted in the same bands (merge_lower_tier)
    const from = MATERIALS[label].from;
    const source =
      from && enabled[label] !== false && enabled[from.label] !== false
        ? from
        : null;
    const ratio = source?.ratio ?? 1;
    // bound, then roster-bound (tradable T4 converts roster-bound), then tradable
    let band = spend_band(
      bound_owned[label],
      source ? [bound_owned[source.label]] : [],
      ratio,
      Math.max(0, cost[label] ?? 0),
    );
    bound_budgets[label] = band.own;
    if (source) [bound_budgets[source.label]] = band.sources;
    band = spend_band(
      roster_owned[label],
      source ? [roster_owned[source.label], tradable_owned[source.label]] : [],
      ratio,
      band.cost,
    );
    roster_mats[label] = band.own;
    if (source)
      [roster_mats[source.label], tradable_mats[source.label]] = band.sources;
    tradable_mats[label] = spend_band(
      tradable_owned[label],
      [],
      1,
      band.cost,
    ).own;
  }
  return { bound_budgets, roster_mats, tradable_mats };
}

// Spends a band: the tier's own stock first, as the user sees it in game, then the lower-tier
// sources at the ratio in whole conversions. What is left of each, and of the cost.
export function spend_band(
  own: number,
  sources: number[],
  ratio: number,
  cost: number,
) {
  const deduct = Math.min(own, cost);
  cost -= deduct;
  const total = sources.reduce((a, b) => a + b, 0);
  const units = Math.min(Math.floor(total / ratio), cost);
  cost -= units;
  let take = units * ratio;
  return {
    own: Math.max(0, own - deduct),
    sources: sources.map((amount) => {
      const taken = Math.min(amount, take);
      take -= taken;
      return amount - taken;
    }),
    cost,
  };
}

// the character's free taps, a Serca one's T4 special leaps counted
export function special_total() {
  const { active_profile } = storeToRefs(useRosterStore());
  const tier = active_profile.value.tier;
  return merge_lower_tier(
    tier,
    input_column_to_num(active_profile.value.special_budget),
    {},
    {},
  ).bound[SPECIAL_LEAP_LABELS[tier]];
}

// free taps left after special honing, spent like any other material
export function set_special_remaining(remaining: number) {
  const { active_profile } = storeToRefs(useRosterStore());
  const label = SPECIAL_LEAP_LABELS[active_profile.value.tier];
  const column = active_profile.value.special_budget;
  const owned = input_column_to_num(column);
  const from = MATERIALS[label].from;
  const left = spend_band(
    owned[label],
    from ? [owned[from.label]] : [],
    from?.ratio ?? 1,
    Math.max(0, special_total() - remaining),
  );
  column.values[label] = left.own.toLocaleString();
  if (from) column.values[from.label] = left.sources[0].toLocaleString();
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

  const total_used: Record<string, number> = {};
  for (const label of ALL_LABELS[tier]) {
    total_used[label] = Object.values(active_profile.value.keyed_upgrades)
      .map((u) => u.used_materials?.[label] ?? 0)
      .reduce((acc, cur) => acc + cur, 0);
  }
  const remaining: RemainingMats = compute_remaininig_materials(
    total_used,
    runtime.budget_snapshot,
  );

  // the tier's labels and any lower-tier sources spent with them
  const write = (column: InputColumn, values: Record<string, number>) => {
    for (const [label, value] of Object.entries(values)) {
      if (column.enabled[label]) {
        column.values[label] = value.toLocaleString();
      }
    }
  };
  write(active_profile.value.bound_budgets, remaining.bound_budgets);
  write(active_roster_mats_owned.value, remaining.roster_mats);
  write(active_tradable_mats_owned.value, remaining.tradable_mats);
}
