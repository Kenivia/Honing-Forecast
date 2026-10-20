import { useRosterStore } from "@/Stores/RosterConfig";
import {
  ALL_LABELS,
  MATERIALS,
  NUM_BASE_MATS,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import { merge_lower_tier } from "@/WasmInterface/PayloadBuilder";
import { input_column_to_num, InputColumn } from "@/Utils/InputColumn";
import {
  OneUpgradeInput,
  to_upgrade_key,
  Upgrade,
  UpgradeStatus,
} from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { nextTick, toRaw } from "vue";
import { grid_change_callback } from "@/Components/Character/CharWorkerUtils";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import type { CharProfile } from "@/Stores/CharacterProfile";

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
// What the taps since the last run cost; the unlock too unless already unlocked (Rust's rule).
// Normal honing only: advanced honing is never deducted.
export function compute_used_materials(
  upgrade: Upgrade,
  taps: number,
  juice_info: any,
  pretend_zero_no_unlock: boolean,
  tier: number,
): Record<string, number> {
  const labels = ALL_LABELS[tier];
  const out: Record<string, number> = Object.fromEntries(
    labels.map((label) => [label, 0]),
  );
  const unlocked =
    upgrade.starting_artisan > 0 || upgrade.starting_num_taps > 0;
  const unlock = (pretend_zero_no_unlock && taps === 0) || unlocked ? 0 : 1;
  for (let row = 0; row < NUM_BASE_MATS; row++) {
    out[labels[row]] =
      upgrade.unlock_costs[row] * unlock + upgrade.costs[row] * taps;
  }
  // the pity tap takes no juice
  const juiced = upgrade.state.slice(
    0,
    Math.min(taps, upgrade.normal_dist.length - 2),
  );
  const ids =
    juice_info.normal_uindex_to_id[upgrade.piece_type_usize][
      upgrade.upgrade_index
    ];
  for (const id of ids) {
    const amt = juice_info.all_juices[id][upgrade.piece_type_usize].get(
      upgrade.upgrade_index,
    ).normal_amt_used;
    out[labels[NUM_BASE_MATS + id]] =
      juiced.filter((one) => one.includes(id)).length * amt;
  }
  return out;
}
export function make_budget_snapshot(): BudgetSnapshot {
  const {
    active_profile,
    active_roster_mats_owned,
    active_tradable_mats_owned,
  } = storeToRefs(useRosterStore());
  return {
    bound_budgets: structuredClone(toRaw(active_profile.value.bound_budgets)),
    roster_mats: structuredClone(toRaw(active_roster_mats_owned.value)),
    tradable_mats: structuredClone(toRaw(active_tradable_mats_owned.value)),
  };
}
export function compute_remaining_materials(
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
  const id = active_profile.value.id;
  runtime.budget_snapshots[id] ??= make_budget_snapshot();

  const total_used: Record<string, number> = {};
  for (const label of ALL_LABELS[tier]) {
    total_used[label] = Object.values(active_profile.value.keyed_upgrades)
      .map((u) => u.used_materials?.[label] ?? 0)
      .reduce((acc, cur) => acc + cur, 0);
  }
  const remaining: RemainingMats = compute_remaining_materials(
    total_used,
    runtime.budget_snapshots[id],
  );

  write_remaining(active_profile.value.bound_budgets, remaining.bound_budgets);
  write_remaining(active_roster_mats_owned.value, remaining.roster_mats);
  write_remaining(active_tradable_mats_owned.value, remaining.tradable_mats);
}

// the tier's labels and any lower-tier sources spent with them; disabled cells are left alone
function write_remaining(column: InputColumn, values: Record<string, number>) {
  for (const [label, value] of Object.entries(values)) {
    if (column.enabled[label]) column.values[label] = value.toLocaleString();
  }
}

// Deductions so far are final: the next ones start from the budgets as they are now.
export function clear_snapshot() {
  delete useRuntimeStore().budget_snapshots[useRosterStore().active_profile.id];
}

// Leaving a character's calculator settles its deductions as Confirm does: the budgets keep them
// and the progress is already written, so another page's or character's edits are never reverted.
export function settle_deductions(profile: CharProfile) {
  for (const keyed of Object.values(profile.keyed_upgrades)) {
    keyed.taps_since_last_input = 0;
    keyed.used_materials = null;
  }
  delete useRuntimeStore().budget_snapshots[profile.id];
}

// Sets one upgrade's usage and deducts. With nothing outstanding the budgets are the truth,
// so the snapshot is retaken (it may predate an edit or another character's switch).
export function deduct(
  keyed: OneUpgradeInput,
  used: Record<string, number> | null,
) {
  const outstanding = Object.values(
    useRosterStore().active_profile.keyed_upgrades,
  ).some(
    (u) => u.used_materials && Object.values(u.used_materials).some((v) => v),
  );
  if (!outstanding) clear_snapshot();
  keyed.used_materials = used;
  apply_remaining_mats();
}

// Succeeded after `taps` taps since the last run: deducts them and the unlock, folds that into
// the snapshot so other upgrades' deductions still count from it, then drops the upgrade.
export function finish_upgrade(
  upgrade: Upgrade,
  taps: number,
  juice_info: any,
) {
  const runtime = useRuntimeStore();
  const profile = useRosterStore().active_profile;
  const keyed =
    profile.keyed_upgrades[
      to_upgrade_key(
        upgrade.piece_index,
        upgrade.upgrade_index,
        upgrade.is_normal_honing,
        profile.tier,
      )
    ];
  runtime.is_details_update = true;
  const used = compute_used_materials(
    upgrade,
    taps,
    juice_info,
    false,
    profile.tier,
  );
  deduct(keyed, used);
  const snapshot = runtime.budget_snapshots[profile.id];
  if (snapshot) {
    const left = compute_remaining_materials(used, snapshot);
    write_remaining(snapshot.bound_budgets, left.bound_budgets);
    write_remaining(snapshot.roster_mats, left.roster_mats);
    write_remaining(snapshot.tradable_mats, left.tradable_mats);
  }
  keyed.used_materials = null;
  keyed.taps_since_last_input = 0;
  mark_upgrade_as_done(upgrade);
  nextTick(() => {
    runtime.is_details_update = false;
  });
}
