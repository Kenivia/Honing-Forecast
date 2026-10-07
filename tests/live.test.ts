// Runs the V7 -> V8 migration against the real save in scripts/V7 config.
// V8 keys every material value by label and stores a shared material once, so the old
// per-tier columns are merged: the character's active tier wins for its own budgets, and
// tier 0 wins for the roster-wide columns that V7 kept in sync.
import fs from "node:fs";
import { check, done } from "./helpers";
import LZString from "lz-string";
import {
  ALL_LABELS,
  ALL_MATERIAL_LABELS,
  bundle_key,
  CONVERTIBLE_MATERIALS,
  effective_unit_prices,
  MATERIALS,
  SHARED_LABELS,
  SPECIAL_LEAP_LABELS,
  unit_prices,
} from "@/Utils/Constants";
import {
  load_roster_config,
  export_config,
  import_config,
  STORAGE_KEY,
  STORAGE_VERSION,
  write_state,
} from "@/Stores/ConfigStorage";
import { to_saved, from_saved } from "@/Stores/SavedConfig";
import { input_column_to_num, parse_locale_int } from "@/Utils/InputColumn";

const FIXTURE = "scripts/V7 config";
if (!fs.existsSync(FIXTURE)) {
  console.log(`  skipped: ${FIXTURE} is not present`);
  process.exit(0);
}
const compressed = fs.readFileSync(FIXTURE, "utf8");
const v7 = JSON.parse(LZString.decompressFromUTF16(compressed));

localStorage.clear();
localStorage.setItem("HF_CONFIG_V7_COMPRESSED", compressed);
const loaded = load_roster_config();

console.log(
  `\n=== live save: ${v7.profiles.length} profiles, ${Object.keys(v7.roster_mats_owned).length} rosters ===`,
);
check("all profiles survived", loaded.profiles.length === v7.profiles.length);
check(
  "names unchanged",
  loaded.profiles.map((p) => p.char_name).join(",") ===
    v7.profiles.map((p: any) => p.char_name).join(","),
);
check(
  "tiers unchanged",
  loaded.profiles.every((p, i) => p.tier === v7.profiles[i].tier),
);
check(
  "roster ids unchanged",
  loaded.profiles.every((p, i) => p.roster_id === v7.profiles[i].roster_id),
);
check(
  "rosters unchanged",
  Object.keys(loaded.roster_mats_owned).sort().join(",") ===
    Object.keys(v7.roster_mats_owned).sort().join(","),
);
check(
  "active index kept",
  loaded.active_profile_index === v7.active_profile_index,
);

// The expected value for every label, read POSITIONALLY from the old per-tier columns,
// with `winner` applied last so it decides shared labels.
function expected(old_cols: any, winner: number) {
  const values: Record<string, number> = {};
  const enabled: Record<string, boolean> = {};
  const order = [0, 1].filter((t) => t !== winner).concat(winner);
  for (const tier of order) {
    ALL_LABELS[tier].forEach((label, row) => {
      const n = parse_locale_int(String(old_cols?.[tier]?.data?.[row] ?? "0"));
      values[label] = Number.isFinite(n) ? n : 0;
      enabled[label] = old_cols?.[tier]?.enabled?.[row] !== false;
    });
  }
  return { values, enabled };
}

let compared = 0;
function compare_merged(
  name: string,
  old_cols: any,
  new_col: any,
  winner: number,
) {
  const want = expected(old_cols, winner);
  const got = input_column_to_num(new_col, true);
  const bad: string[] = [];
  const bad_en: string[] = [];
  for (const label of Object.keys(want.values)) {
    compared++;
    if (got[label] !== want.values[label]) {
      bad.push(`${label}: ${got[label]} != ${want.values[label]}`);
    }
    if (new_col.enabled[label] !== want.enabled[label]) {
      bad_en.push(label);
    }
  }
  check(name, bad.length === 0, bad.slice(0, 3).join("; "));
  check(name + " (enabled)", bad_en.length === 0, bad_en.slice(0, 5).join(","));
}

console.log("\n--- per-character budgets, positional reading vs migrated");
loaded.profiles.forEach((p, i) => {
  const o = v7.profiles[i];
  compare_merged(
    `  ${p.char_name} bound_budgets`,
    o.bound_budgets,
    p.bound_budgets,
    p.tier,
  );
  compare_merged(
    `  ${p.char_name} leftover_price`,
    o.leftover_price,
    p.leftover_price,
    p.tier,
  );
  // the single V7 special-leap number belongs to the character's current tier
  const want = parse_locale_int(String(o.special_budget?.data?.[0] ?? "0"));
  const got = input_column_to_num(p.special_budget, true);
  check(
    `  ${p.char_name} special_budget -> ${SPECIAL_LEAP_LABELS[p.tier]}`,
    got[SPECIAL_LEAP_LABELS[p.tier]] === (Number.isFinite(want) ? want : 0),
  );
});

console.log(
  "\n--- roster-owned and tradable (tier 0 was the authoritative copy)",
);
for (const roster_id of Object.keys(v7.roster_mats_owned)) {
  compare_merged(
    `  roster ${roster_id} owned`,
    v7.roster_mats_owned[roster_id],
    loaded.roster_mats_owned[roster_id],
    0,
  );
  compare_merged(
    `  roster ${roster_id} tradable`,
    v7.tradable_mats_owned[roster_id],
    loaded.tradable_mats_owned[roster_id],
    0,
  );
}

console.log("\n--- market prices, now keyed by (material, bundle size)");
for (const region of Object.keys(v7.mats_prices)) {
  const got = input_column_to_num(loaded.mats_prices[region], true);
  const want = expected(v7.mats_prices[region], 0);
  const bad: string[] = [];
  for (const label of ALL_MATERIAL_LABELS) {
    const mat = MATERIALS[label];
    if (mat.bundle_sizes.length > 1) continue; // shard bags checked below
    if (want.values[label] === undefined) continue;
    compared++;
    const key = bundle_key(label, mat.bundle_sizes[0]);
    if (got[key] !== want.values[label]) {
      bad.push(`${key}: ${got[key]} != ${want.values[label]}`);
    }
  }
  check(`  prices ${region}`, bad.length === 0, bad.slice(0, 3).join("; "));

  // the old shard_infos structure folds into Shards' three bundles
  const old_shards = v7.shard_infos?.[region];
  const bad_bags: string[] = [];
  for (const size of Object.keys(old_shards?.prices ?? {})) {
    compared++;
    const w = parse_locale_int(String(old_shards.prices[size].data[0]));
    const g = got[bundle_key("Shards", Number(size))];
    if (g !== w) bad_bags.push(`x${size}: ${g} != ${w}`);
  }
  check(
    `  shard bag prices ${region}`,
    bad_bags.length === 0,
    bad_bags.join("; "),
  );
  check(
    `  shard bag selection ${region}`,
    loaded.selected_bundles[region].Shards === old_shards.selected,
    `${loaded.selected_bundles[region].Shards} vs ${old_shards.selected}`,
  );
}

console.log("\n--- shared materials are stored once, not once per tier");
for (const label of SHARED_LABELS) {
  check(
    `  ${label} appears once in a column`,
    Object.keys(loaded.roster_mats_owned[0].values).filter((k) => k === label)
      .length === 1,
  );
}
check(
  "a tier-1 character still carries its T4-only labels",
  loaded.profiles.every((p) =>
    ["Red", "Blue", "Leaps", "Fusion"].every(
      (l) => p.bound_budgets.values[l] !== undefined,
    ),
  ),
);
check(
  "every tier label exists in every column",
  loaded.profiles.every((p) =>
    ALL_MATERIAL_LABELS.every((l) => p.bound_budgets.values[l] !== undefined),
  ),
);

console.log("\n--- conversion relations");
const region0 = loaded.all_regions[loaded.profiles[0].roster_id];
const units = unit_prices(
  input_column_to_num(loaded.mats_prices[region0]),
  loaded.selected_bundles[region0],
);
const eff = effective_unit_prices(units);
for (const mat of CONVERTIBLE_MATERIALS) {
  check(
    `  ${mat.label} effective = min(own, ${mat.from.label} x${mat.from.ratio})`,
    eff[mat.label] ===
      Math.min(units[mat.label], units[mat.from.label] * mat.from.ratio),
  );
}
check(
  "shared materials are never converted",
  SHARED_LABELS.every((l) => eff[l] === units[l]),
);
check(
  "shard unit price = bag price / bag size",
  units["Shards"] ===
    input_column_to_num(loaded.mats_prices[region0])[
      bundle_key("Shards", loaded.selected_bundles[region0].Shards)
    ] /
      loaded.selected_bundles[region0].Shards,
);

console.log("\n--- grids and keyed upgrades");
let grid_ok = true;
let keyed_ok = true;
let keyed_total = 0;
loaded.profiles.forEach((p, i) => {
  const o = v7.profiles[i];
  if (JSON.stringify(p.adv_grid) !== JSON.stringify(o.adv_grid))
    grid_ok = false;
  const floor = p.tier === 0 ? 10 : 11;
  p.normal_grid.forEach((row, piece) => {
    if (piece === 6) return;
    row.forEach((cell, col) => {
      if (col >= floor && cell !== o.normal_grid[piece][col]) grid_ok = false;
    });
  });
  for (const [key, u] of Object.entries(o.keyed_upgrades ?? {})) {
    keyed_total += 1;
    const got = p.keyed_upgrades[key];
    if (!got) continue;
    const want: any = u;
    if (
      got.starting_artisan !== want.starting_artisan ||
      got.starting_num_taps !== want.starting_num_taps ||
      JSON.stringify(got.state) !== JSON.stringify(want.state) ||
      JSON.stringify(got.adv_progress) !== JSON.stringify(want.adv_progress)
    ) {
      keyed_ok = false;
    }
  }
});
check("grids preserved above the tier floor", grid_ok);
check(`keyed upgrade progress preserved (${keyed_total} entries)`, keyed_ok);

console.log("\n--- transient fields not resurrected");
for (const key of [
  "tier",
  "is_fetching",
  "market_fetch_failed",
  "is_details_update",
  "effective_serca_price",
  "shard_infos",
]) {
  check(`  ${key} absent`, !(key in loaded));
}

console.log("\n--- round trip and export/import on the real data");
const once = to_saved(loaded);
const twice = to_saved(from_saved({ ...once, version: STORAGE_VERSION }));
check(
  "to_saved is a fixed point",
  JSON.stringify(once) === JSON.stringify(twice),
);
check(
  "export -> import is lossless",
  JSON.stringify(to_saved(import_config(export_config(loaded)))) ===
    JSON.stringify(once),
);

console.log("\n--- load, save, load again on the real data");
// The write path and the V8 read path are separate code, and a save that cannot be read
// back identically only shows up on a second load.
write_state({ roster_config: loaded });
const written = localStorage.getItem(STORAGE_KEY);
check("written under the current key", written !== null);
const second = load_roster_config();
check(
  "second load keeps all 27 characters",
  second.profiles.map((p) => p.char_name).join(",") ===
    loaded.profiles.map((p) => p.char_name).join(","),
);
check(
  "second load is byte-identical to the first",
  JSON.stringify(to_saved(second)) === JSON.stringify(once),
);
write_state({ roster_config: second });
const third = load_roster_config();
check(
  "third load is still identical",
  JSON.stringify(to_saved(third)) === JSON.stringify(once),
);

console.log("\n--- size");
const v7_json = JSON.stringify(v7);
const v8_json = JSON.stringify({ version: STORAGE_VERSION, ...once });
console.log(
  `  json       ${v7_json.length} B -> ${v8_json.length} B  (${Math.round((1 - v8_json.length / v7_json.length) * 100)}% smaller)`,
);
console.log(
  `  in localStorage ${compressed.length} -> ${LZString.compressToUTF16(v8_json).length} UTF16 chars`,
);

done(`${compared} material values compared`);
