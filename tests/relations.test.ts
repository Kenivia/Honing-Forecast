import { check, done, equal } from "./helpers";
// Covers the generalized cross-tier relations and bundle pricing.
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { change_tier } from "@/Components/Character/StatusInput/StatusInputUtil";
import {
  compute_remaining_materials,
  set_special_remaining,
  special_total,
} from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";
import { column_from_saved } from "@/Stores/SavedConfig";
import {
  ALL_LABELS,
  ALL_MATERIAL_LABELS,
  bundle_key,
  cheapest_bundles,
  linked_labels,
  CONVERTIBLE_MATERIALS,
  MATERIALS,
  SHARED_LABELS,
  SPECIAL_LEAP_LABELS,
  unit_prices,
} from "@/Utils/Constants";
import {
  create_input_column,
  input_column_to_num,
  InputType,
  set_cell,
} from "@/Utils/InputColumn";
import {
  build_materials,
  merge_lower_tier,
} from "@/WasmInterface/PayloadBuilder";

setActivePinia(createPinia());
const store = useRosterStore();

console.log("\n=== 1. change_tier leaves the budgets alone ===");
const p = store.active_profile;
// a distinct value per material so a mix-up is visible
ALL_MATERIAL_LABELS.forEach((label, i) =>
  set_cell(p.bound_budgets, label, String((i + 1) * 1000)),
);
set_cell(p.special_budget, SPECIAL_LEAP_LABELS[0], "500");
const before = input_column_to_num(p.bound_budgets, true);
const before_leaps = input_column_to_num(p.special_budget, true);

change_tier(p);
check("tier moved to 1", p.tier === 1);
equal("bound unchanged", input_column_to_num(p.bound_budgets, true), before);
equal(
  "special leaps unchanged",
  input_column_to_num(p.special_budget, true),
  before_leaps,
);
change_tier(p);
check("tier moved back to 0", p.tier === 0);
equal("still unchanged", input_column_to_num(p.bound_budgets, true), before);

console.log("\n=== 2. bundle pricing ===");
const prices: Record<string, number> = {};
for (const label of ALL_MATERIAL_LABELS) {
  MATERIALS[label].bundle_sizes.forEach((size) => {
    prices[bundle_key(label, size)] = size * 2; // 2 gold per unit, every bundle
  });
}
const selected = cheapest_bundles(prices);
const units = unit_prices(prices, selected);
check(
  "a flat per-unit price gives the same unit price for every bundle",
  ALL_MATERIAL_LABELS.every((l) => units[l] === 2),
);
// make the big shard bag the bargain
prices[bundle_key("Shards", 3000)] = 3000;
check(
  "cheapest_bundles picks the best value per unit",
  cheapest_bundles(prices).Shards === 3000,
);
prices[bundle_key("Shards", 1000)] = 100;
check(
  "cheapest_bundles follows a cheaper small bag",
  cheapest_bundles(prices).Shards === 1000,
);
check(
  "single-bundle materials always select their one size",
  ALL_MATERIAL_LABELS.filter(
    (l) => MATERIALS[l].bundle_sizes.length === 1,
  ).every((l) => cheapest_bundles(prices)[l] === MATERIALS[l].bundle_sizes[0]),
);

console.log("\n=== 3. material deduction still covers every material ===");
const tier = store.active_profile.tier;
const labels = ALL_LABELS[tier];
const col = (v: number) =>
  create_input_column(InputType.Int, ALL_MATERIAL_LABELS, {
    value: () => String(v),
  });
const snapshot = {
  bound_budgets: col(10),
  roster_mats: col(10),
  tradable_mats: col(10),
} as any;
const used = (v: number): Record<string, number> =>
  Object.fromEntries(labels.map((l) => [l, v]));
const out = compute_remaining_materials(used(25), snapshot);
const wrong = labels.filter(
  (l) =>
    out.bound_budgets[l] !== 0 ||
    out.roster_mats[l] !== 0 ||
    out.tradable_mats[l] !== 5,
);
check(
  `all ${labels.length} materials deduct through all three pools`,
  wrong.length === 0,
  wrong.slice(0, 4).join(","),
);
const zero = compute_remaining_materials(used(0), snapshot);
check(
  "zero cost leaves every pool untouched",
  labels.every(
    (l) =>
      zero.bound_budgets[l] === 10 &&
      zero.roster_mats[l] === 10 &&
      zero.tradable_mats[l] === 10,
  ),
);

console.log("\n=== 4. a Serca character's T4 stock ===");
const t4 = (b: number, r: number, t: number, sb = 0, sr = 0, st = 0) =>
  [
    { Fusion: b, "Serca Fusion": sb },
    { Fusion: r, "Serca Fusion": sr },
    { Fusion: t, "Serca Fusion": st },
  ] as const;
const m = merge_lower_tier(1, ...t4(12, 3, 4, 1, 2, 3));
equal(
  "bound into bound, roster and tradable pooled into roster, floored",
  [
    m.bound["Serca Fusion"],
    m.roster["Serca Fusion"],
    m.tradable["Serca Fusion"],
  ],
  [1 + 2, 2 + 1, 3],
);
equal("T4 itself left as it is", [m.bound.Fusion, m.tradable.Fusion], [12, 4]);
equal(
  "tradable T4 alone counts roster-bound",
  merge_lower_tier(1, ...t4(0, 0, 10)).roster["Serca Fusion"],
  2,
);
equal(
  "nothing for a T4 character",
  merge_lower_tier(0, ...t4(12, 3, 4)).bound["Serca Fusion"],
  0,
);
equal(
  "a disabled source is left out",
  merge_lower_tier(1, ...t4(12, 3, 4), { Fusion: false }).bound["Serca Fusion"],
  0,
);

p.tier = 1;
const serca = (values: Record<string, number>) =>
  create_input_column(InputType.Int, ALL_MATERIAL_LABELS, {
    value: (label) => String(values[label] ?? 0),
  });
set_cell(p.bound_budgets, "Fusion", "7");
set_cell(p.bound_budgets, "Serca Fusion", "1");
set_cell(store.active_roster_mats_owned, "Fusion", "3");
set_cell(store.active_roster_mats_owned, "Serca Fusion", "2");
set_cell(store.active_tradable_mats_owned, "Fusion", "4");
set_cell(store.active_tradable_mats_owned, "Serca Fusion", "3");
const sent = build_materials()["Serca Fusion"];
equal(
  "payload: T4 bound into bound, roster-bound and tradable into roster",
  [sent.bound, sent.roster, sent.tradable],
  [1 + 1, 2 + 1, 3],
);

const serca_snapshot = {
  bound_budgets: serca({ Fusion: 7, "Serca Fusion": 1 }),
  roster_mats: serca({ Fusion: 3, "Serca Fusion": 2 }),
  tradable_mats: serca({ Fusion: 4, "Serca Fusion": 3 }),
};
const left = compute_remaining_materials(
  { "Serca Fusion": 6 },
  serca_snapshot,
);
const of = (label: string) => [
  left.bound_budgets[label],
  left.roster_mats[label],
  left.tradable_mats[label],
];
// bound band: the 1 Serca, then 1 from T4 (5 Fusion); roster band: the 2 Serca, then 1 from T4
// roster and tradable pooled (3 + 2 Fusion); the last 1 from Serca tradable
equal("deduction: T4 after Serca in each band", of("Fusion"), [2, 0, 2]);
equal("deduction: the Serca stock first", of("Serca Fusion"), [0, 0, 2]);
const small = compute_remaining_materials(
  { "Serca Fusion": 1 },
  serca_snapshot,
);
equal(
  "a small cost comes out of Serca bound alone",
  [small.bound_budgets.Fusion, small.bound_budgets["Serca Fusion"]],
  [7, 0],
);
// free taps: Serca first, then T4 special leaps in whole conversions
p.tier = 1;
set_cell(p.special_budget, "Serca Special Leap", "2");
set_cell(p.special_budget, "Special Leap", "12");
equal("free taps count T4", special_total(), 4);
set_special_remaining(1);
equal(
  "free taps spent Serca first",
  [
    input_column_to_num(p.special_budget)["Serca Special Leap"],
    input_column_to_num(p.special_budget)["Special Leap"],
  ],
  [0, 7],
);
set_special_remaining(0);
equal("all failed", special_total(), 0);

// a material and its other tier share the tick, saved or not
equal("linked labels", linked_labels("Fusion"), ["Fusion", "Serca Fusion"]);
const saved_column = column_from_saved(
  { values: {}, disabled: ["Serca Red"] },
  p.bound_budgets,
);
check("loading disables both", !saved_column.enabled.Red);

serca_snapshot.bound_budgets.enabled.Fusion = false;
const off = compute_remaining_materials({ "Serca Fusion": 1 }, serca_snapshot);
check(
  "a disabled source is not spent",
  off.bound_budgets.Fusion === undefined &&
    off.bound_budgets["Serca Fusion"] === 0,
);
p.tier = 0;

done();
