import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { AdvOverride, NormalOverride } from "@/WasmInterface/Overrides";
import { StateBundle } from "@/WasmInterface/WasmTypes";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import {
  BAND_NAMES,
  clamp_band_plan,
  JOINED_ADV_JUICE,
  NUM_BASE_MATS,
} from "@/Utils/Constants";
import { artisan_number, artisan_string } from "@/Utils/HoningUtil";
import { toOrdinal } from "@/Utils/Helpers";

export function get_any_overwritten(): boolean {
  const { optimizer_override } = useRosterStore().active_profile;
  return (
    optimizer_override.normal.juice !== NormalOverride.Optimizer ||
    optimizer_override.normal.book !== NormalOverride.Optimizer ||
    optimizer_override.advanced.juice !== AdvOverride.Optimizer ||
    optimizer_override.advanced.scroll !== AdvOverride.Optimizer ||
    optimizer_override.special.optimizer !== true
  );
}

// When no plan can cost gold: every base mat used is disabled, or covered at no-juice pity by
// the bands worth nothing.
export function get_zero_gold_warning(): string | null {
  const result = useRuntimeStore().histogram.result;
  if (!result) return null;
  const { band_values, bound_budgets } = useRosterStore().active_profile;
  const { table, materials } = result.state_bundle.prep_output;
  const rows = table.labels
    .slice(0, NUM_BASE_MATS)
    .map((label: string, row: number) => [label, row])
    .filter(([label]) => result.no_juice_pity?.[label] > 0);
  if (rows.length === 0) return null;
  const disabled = (label: string) => bound_budgets.enabled[label] === false;
  if (rows.every(([label]) => disabled(label)))
    return "You've disabled every mat, so no matter how you juice, it costs 0 gold. You should probably enable something";
  const worthless = clamp_band_plan(band_values).filter(
    (v) => v === "Worthless",
  ).length;
  const covered = ([label, row]) => {
    const { bound, roster, tradable } = materials[row];
    const owned = [bound, roster, tradable].slice(0, worthless);
    return owned.reduce((a, b) => a + b, 0) >= result.no_juice_pity[label];
  };
  if (worthless > 0 && rows.every((r) => disabled(r[0]) || covered(r)))
    return `You can afford pity with ${BAND_NAMES.slice(0, worthless).join(" + ").toLowerCase()} mats, so no matter how you juice, it costs 0 gold.`;
  return null;
}

export function get_optimizer_working(): boolean {
  return useRuntimeStore().optimizer.status === "busy";
}

// the histogram's bundle evaluates the overrides; the optimizer's is the plan
export function get_relevant_result(any_overwritten: boolean): StateBundle {
  const runtime = useRuntimeStore();
  return any_overwritten
    ? runtime.histogram.result?.state_bundle
    : runtime.optimizer.result;
}

// [upgrade with its free-tap chance, index in special_state, perform order]
export type InstructionRow = [Upgrade, number, number];

// An order that can be done in game: the free-tap order, each upgrade after the lower levels
// of its piece, then the rest. One of many equivalent orders.
export function sort_upgrades(bundle: StateBundle | null): InstructionRow[] {
  if (!bundle) return [];
  const { upgrade_arr, special_state, special_invalid_index } = bundle;
  const order: number[] = [];
  for (const [pos, index] of special_state.entries()) {
    if (pos >= special_invalid_index) {
      if (!order.includes(index)) order.push(index);
      continue;
    }
    const target = upgrade_arr[index];
    for (const [i, u] of upgrade_arr.entries()) {
      if (
        u.is_normal_honing &&
        u.piece_index === target.piece_index &&
        u.upgrade_index <= target.upgrade_index &&
        !u.succeeded &&
        !order.includes(i)
      )
        order.push(i);
    }
  }
  return order.map((index, perform_order) => {
    const pos = special_state.indexOf(index);
    const this_special_chance = bundle.latest_special_probs[pos];
    return [{ ...upgrade_arr[index], this_special_chance }, pos, perform_order];
  });
}

// One section's rows; advanced ones by level.
export function instruction_rows(
  bundle: StateBundle | null,
  is_normal: boolean,
): InstructionRow[] {
  const out = sort_upgrades(bundle).filter(
    ([u]) => u.is_normal_honing === is_normal,
  );
  if (!is_normal) out.sort(([a], [b]) => a.upgrade_index - b.upgrade_index);
  return out;
}

// A run of taps that take the same ids. Advanced counts are 255 for "all".
export interface Streak {
  ids: number[];
  count: number;
  grace: boolean | null;
  pity: boolean;
}

// Equal taps grouped. The pity tap takes nothing, but Rust does not empty its slot, so it is
// shown on its own (not for an upgrade that always succeeds, which never reaches artisan).
export function normal_streaks(upgrade: Upgrade, juice_info: any): Streak[] {
  const out: Streak[] = [];
  const pity_index = upgrade.normal_dist.length - 2;
  const states = upgrade.state.slice(0, upgrade.normal_dist.length);
  for (const [index, ids] of states.entries()) {
    if (
      index === pity_index &&
      artisan_number(upgrade, index, juice_info) >= 1
    ) {
      out.push({ ids: [], count: 1, grace: null, pity: true });
      break;
    }
    const last = out.at(-1);
    if (last && same_ids(last.ids, ids)) last.count++;
    else out.push({ ids, count: 1, grace: null, pity: false });
  }
  return out;
}

function same_ids(a: number[], b: number[]) {
  const set = new Set(b);
  return new Set(a).size === set.size && a.every((id) => set.has(id));
}

// The juice and scroll strategies (first N grace taps, then N non-grace) as streaks.
export function adv_streaks(
  upgrade: Upgrade,
  [juice_id, scroll_id]: number[],
): Streak[] {
  const juice = JOINED_ADV_JUICE[upgrade.state[0][0]];
  const scroll = JOINED_ADV_JUICE[upgrade.state[1][0]];
  const out: Streak[] = [];
  for (const [i, grace] of [
    [0, true],
    [1, false],
  ] as const) {
    const [j, s] = [juice[i], scroll[i]];
    const both = Math.min(j, s);
    if (both > 0)
      out.push({ ids: [juice_id, scroll_id], count: both, grace, pity: false });
    const one =
      j === s ? 0 : Math.max(j, s) === 255 ? 255 : Math.max(j, s) - both;
    if (one > 0)
      out.push({
        ids: [j > s ? juice_id : scroll_id],
        count: one,
        grace,
        pity: false,
      });
  }
  if (out.length === 0)
    out.push({ ids: [], count: 255, grace: true, pity: false });
  return out;
}

// The two text lines under each streak.
export function streak_lines(
  upgrade: Upgrade,
  streaks: Streak[],
  juice_info: any,
): [string, string][] {
  let taps = 0;
  return streaks.map((streak): [string, string] => {
    if (upgrade.is_normal_honing) {
      taps += streak.count;
      return streak.pity
        ? ["Pity", `occurs on the ${toOrdinal(taps)} tap`]
        : [
            `x${streak.count} taps`,
            `until ${artisan_string(upgrade, taps, juice_info)}% artisan`,
          ];
    }
    const grace = streak.grace ? "Grace" : "non-Grace";
    if (streak.ids.length === 0) return ["Nothing", `on ${grace}`];
    if (streak.count === 255) return ["All", grace];
    return [`${streak.grace ? "First" : "Any"} ${streak.count}`, grace];
  });
}
