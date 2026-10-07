import { check, done } from "./helpers";
// Covers the generalized cross-tier relations and bundle pricing.
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { change_tier } from "@/Components/Character/StatusInput/StatusInputUtil";
import { compute_remaininig_materials } from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";
import {
  ALL_LABELS,
  ALL_MATERIAL_LABELS,
  bundle_key,
  cheapest_bundles,
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

setActivePinia(createPinia());
const store = useRosterStore();

console.log("\n=== 1. change_tier converts by relation, not by index ===");
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
const at1 = input_column_to_num(p.bound_budgets, true);
for (const mat of CONVERTIBLE_MATERIALS) {
  if (mat.label.includes("Special Leap")) continue;
  check(
    `  ${mat.label} = ${mat.from.label} / ${mat.from.ratio}`,
    at1[mat.label] === Math.floor(before[mat.from.label] / mat.from.ratio),
    `${at1[mat.label]} vs ${before[mat.from.label] / mat.from.ratio}`,
  );
}
check(
  "shared materials untouched by a tier change",
  SHARED_LABELS.every((l) => at1[l] === before[l]),
);
const leaps1 = input_column_to_num(p.special_budget, true);
check(
  "special leaps convert too",
  leaps1[SPECIAL_LEAP_LABELS[1]] ===
    Math.floor(before_leaps[SPECIAL_LEAP_LABELS[0]] / 5),
  `${leaps1[SPECIAL_LEAP_LABELS[1]]}`,
);

change_tier(p);
check("tier moved back to 0", p.tier === 0);
const at0 = input_column_to_num(p.bound_budgets, true);
for (const mat of CONVERTIBLE_MATERIALS) {
  if (mat.label.includes("Special Leap")) continue;
  check(
    `  ${mat.from.label} restored from ${mat.label}`,
    at0[mat.from.label] === at1[mat.label] * mat.from.ratio,
    `${at0[mat.from.label]} vs ${at1[mat.label] * mat.from.ratio}`,
  );
}
check(
  "shared materials still untouched after the round trip",
  SHARED_LABELS.every((l) => at0[l] === before[l]),
);

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
const out = compute_remaininig_materials(
  labels.map(() => 25),
  snapshot,
);
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
const zero = compute_remaininig_materials(
  labels.map(() => 0),
  snapshot,
);
check(
  "zero cost leaves every pool untouched",
  labels.every(
    (l) =>
      zero.bound_budgets[l] === 10 &&
      zero.roster_mats[l] === 10 &&
      zero.tradable_mats[l] === 10,
  ),
);

done();
