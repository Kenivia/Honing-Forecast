import { check, done } from "./helpers";
// Verifies the V7 -> V8 migration, the save/load round trip and export/import.
import {
  ALL_LABELS,
  ALL_MATERIAL_LABELS,
  bundle_key,
  FALLBACK_PRICES,
  MATERIALS,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import {
  load_roster_config,
  export_config,
  import_config,
  STORAGE_VERSION,
  STORAGE_KEY,
  write_state,
} from "@/Stores/ConfigStorage";
import { to_saved, from_saved } from "@/Stores/SavedConfig";
import { input_column_to_num } from "@/Utils/InputColumn";
import LZString from "lz-string";

// A faithful V7 payload: full InputColumns, positional data, locale-formatted strings.
function v7_col(
  keys: string[],
  values: number[],
  disabled_rows: number[] = [],
) {
  return {
    type: 0,
    keys: keys.slice(),
    data: keys.map((_, i) => (values[i] ?? 0).toLocaleString()),
    upper_bound: keys.map(() => 999999999),
    enabled: keys.map((_, i) => !disabled_rows.includes(i)),
  };
}

const T0 = ALL_LABELS[0];
const T1 = ALL_LABELS[1];
// distinctive values, so any positional mix-up shows up
const bound0 = T0.map((_, i) => (i + 1) * 100);
const bound1 = T1.map((_, i) => (i + 1) * 7);
const owned0 = T0.map((_, i) => (i + 1) * 11);
const price0 = T0.map((_, i) => (i + 1) * 3);
const zeros = (keys: string[]) => keys.map(() => 0);

// UpgradeStatus: Done=0, Want=1, NotYet=2, FetchedDone=3
function grid(rows: number, cols: number, fill = 2) {
  return Array.from({ length: rows }, () => Array(cols).fill(fill));
}
const alpha_normal = grid(7, 25);
alpha_normal[5][20] = 1; // Want, so the keyed upgrade below survives grids_to_keyed

const v7: any = {
  profiles: [
    {
      char_name: "Alpha",
      roster_id: 0,
      tier: 0,
      express_event: true,
      auto_start_optimizer: false,
      lock_fetched_done: false,
      pretend_30_40_x2_grace: true,
      optimizer_treatment_plan: 2,
      histogram_treatment_plan: 1,
      normal_grid: alpha_normal,
      adv_grid: grid(6, 4),
      keyed_upgrades: {
        "5,20,true,0": {
          piece_index: 5,
          upgrade_index: 20,
          is_normal_honing: true,
          starting_artisan: 0.42,
          starting_num_taps: 3,
          state: [[0, 2]],
          unlocked: true,
          adv_progress: null,
          double_balls: false,
          expanded: true,
          taps_since_last_input: 2,
          used_materials: [1, 2, 3],
        },
      },
      special_budget: v7_col(["Special Leap"], [1234]),
      bound_budgets: [v7_col(T0, bound0, [3]), v7_col(T1, bound1, [3])],
      leftover_price: [v7_col(T0, zeros(T0)), v7_col(T1, zeros(T1))],
      min_resolution: 1,
      num_threads: 1,
      metric_type: 1,
      optimizer_worker_bundle: null,
      histogram_worker_bundle: null,
    },
    {
      char_name: "Beta",
      roster_id: 1,
      tier: 1,
      normal_grid: grid(7, 25),
      adv_grid: grid(6, 4),
      keyed_upgrades: {},
      special_budget: v7_col(["Special Leap"], [5]),
      bound_budgets: [
        v7_col(
          T0,
          T0.map(() => 1),
        ),
        v7_col(
          T1,
          T1.map(() => 2),
        ),
      ],
      leftover_price: [v7_col(T0, zeros(T0)), v7_col(T1, zeros(T1))],
    },
  ],
  active_profile_index: 1,
  last_seen_version: "v1.2.8",
  mats_prices: {
    nae: [
      v7_col(T0, price0),
      v7_col(
        T1,
        T1.map((_, i) => i + 1),
      ),
    ],
    euc: [
      v7_col(
        T0,
        T0.map(() => 5),
      ),
      v7_col(
        T1,
        T1.map(() => 6),
      ),
    ],
    Custom: [v7_col(T0, zeros(T0)), v7_col(T1, zeros(T1))],
  },
  roster_mats_owned: {
    0: [
      v7_col(T0, owned0),
      v7_col(
        T1,
        T1.map(() => 4),
      ),
    ],
    1: [
      v7_col(
        T0,
        T0.map(() => 9),
      ),
      v7_col(
        T1,
        T1.map(() => 8),
      ),
    ],
  },
  tradable_mats_owned: {
    0: [
      v7_col(
        T0,
        T0.map(() => 2),
      ),
      v7_col(
        T1,
        T1.map(() => 3),
      ),
    ],
    1: [
      v7_col(
        T0,
        T0.map(() => 1),
      ),
      v7_col(
        T1,
        T1.map(() => 1),
      ),
    ],
  },
  all_regions: { 0: "nae", 1: "euc" },
  shard_infos: {
    nae: {
      selected: 2000,
      prices: {
        1000: v7_col(["Shard"], [111]),
        2000: v7_col(["Shard"], [222]),
        3000: v7_col(["Shard"], [333]),
      },
    },
    euc: {
      selected: 3000,
      prices: {
        1000: v7_col(["Shard"], [1]),
        2000: v7_col(["Shard"], [2]),
        3000: v7_col(["Shard"], [3]),
      },
    },
    Custom: {
      selected: 3000,
      prices: {
        1000: v7_col(["Shard"], [0]),
        2000: v7_col(["Shard"], [0]),
        3000: v7_col(["Shard"], [0]),
      },
    },
  },
  latest_market_data: { nae: [1700000000000, { some: "payload" }] },
  cumulative_graph: false,
  show_all_rows: true,
  auto_deduct_costs: false,
  auto_fetch: false,
  enabled_annotations: [false, true, true, false],
  // fields that must NOT come back
  tier: 0,
  is_fetching: true,
  market_fetch_failed: false,
  budget_snapshot: { junk: true },
  adv_cache: { junk: true },
};

console.log("\n=== 1. V7 -> V8 migration through load_roster_config ===");
localStorage.setItem(
  "HF_CONFIG_V7_COMPRESSED",
  LZString.compressToUTF16(JSON.stringify(v7)),
);
localStorage.setItem("HF_UI_STATE_V3_roster", "should be removed");
const loaded = load_roster_config();

check(
  "legacy V7 key consumed",
  localStorage.getItem("HF_CONFIG_V7_COMPRESSED") === null,
);
check(
  "abandoned V3 key removed",
  localStorage.getItem("HF_UI_STATE_V3_roster") === null,
);
check(
  "2 profiles",
  loaded.profiles.length === 2,
  String(loaded.profiles.length),
);
check(
  "names kept",
  loaded.profiles.map((p) => p.char_name).join(",") === "Alpha,Beta",
  loaded.profiles.map((p) => p.char_name).join(","),
);
check("active index kept", loaded.active_profile_index === 1);
check("last_seen_version kept", loaded.last_seen_version === "v1.2.8");
check("tier kept (Beta=1)", loaded.profiles[1].tier === 1);
check("express_event kept", loaded.profiles[0].express_event === true);
check(
  "auto_start_optimizer kept",
  loaded.profiles[0].auto_start_optimizer === false,
);
check(
  "treatment plans kept",
  loaded.profiles[0].optimizer_treatment_plan === 2 &&
    loaded.profiles[0].histogram_treatment_plan === 1,
);
check(
  "roster flags kept",
  loaded.show_all_rows === true &&
    loaded.cumulative_graph === false &&
    loaded.auto_deduct_costs === false &&
    loaded.auto_fetch === false,
);
check(
  "enabled_annotations kept",
  JSON.stringify(loaded.enabled_annotations) === "[false,true,true,false]",
);
check(
  "latest_market_data kept",
  loaded.latest_market_data.nae?.[0] === 1700000000000,
);

console.log("\n--- values land on the right LABEL, not the right index");
// profile 0 is on tier 0, so tier 0 wins for a material shared between tiers
const bb0 = input_column_to_num(loaded.profiles[0].bound_budgets, true);
const bad_t0 = T0.filter((label, i) => bb0[label] !== bound0[i]);
check(
  "tier0 bound budgets per label",
  bad_t0.length === 0,
  bad_t0
    .slice(0, 3)
    .map((l) => `${l}: ${bb0[l]}`)
    .join("; "),
);
// labels that exist only in tier 1 came from the old tier 1 column
const t1_only = T1.filter((l) => !T0.includes(l));
check(
  "tier1-only labels keep their own values",
  t1_only.every((l) => bb0[l] === bound1[T1.indexOf(l)]),
  JSON.stringify(t1_only.map((l) => [l, bb0[l]])),
);
check(
  "a shared label takes the active tier's value",
  bb0["Shards"] === bound0[T0.indexOf("Shards")],
  `${bb0["Shards"]} vs ${bound0[T0.indexOf("Shards")]}`,
);
// Beta is on tier 1 and its fixture is all 1s for tier 0, all 2s for tier 1, so a
// shared label must come out as 2 and a tier-0-only label as 1
const bb_p1 = input_column_to_num(loaded.profiles[1].bound_budgets, true);
check(
  "a tier-1 character takes tier 1's shared value",
  bb_p1["Shards"] === 2,
  String(bb_p1["Shards"]),
);
check(
  "a tier-1 character keeps tier-0-only labels from tier 0",
  T0.filter((l) => !T1.includes(l)).every((l) => bb_p1[l] === 1),
);

const owned = input_column_to_num(loaded.roster_mats_owned[0], true);
check(
  "roster owned per label",
  T0.every((l, i) => owned[l] === owned0[i]),
);
// prices moved to (material, bundle size) keys
const prices = input_column_to_num(loaded.mats_prices.nae, true);
check(
  "nae prices per label, keyed by bundle",
  T0.filter((l) => MATERIALS[l].bundle_sizes.length === 1).every(
    (l) =>
      prices[bundle_key(l, MATERIALS[l].bundle_sizes[0])] ===
      price0[T0.indexOf(l)],
  ),
);
check(
  "special budget kept, under the active tier's label",
  input_column_to_num(loaded.profiles[0].special_budget, true)[
    SPECIAL_LEAP_LABELS[0]
  ] === 1234,
);
check(
  "a tier-1 character's special leaps use the Serca label",
  input_column_to_num(loaded.profiles[1].special_budget, true)[
    SPECIAL_LEAP_LABELS[1]
  ] === 5,
);

console.log("\n--- derived fields come from Constants, not from the save");
check(
  "one column covers every material label",
  Object.keys(loaded.profiles[0].bound_budgets.values).join("|") ===
    ALL_MATERIAL_LABELS.join("|"),
);
check(
  "upper_bound from Constants",
  Object.values(loaded.profiles[0].bound_budgets.upper_bound).every(
    (x) => x === 999999999,
  ),
);
check(
  "special_budget bound is 33333",
  loaded.profiles[0].special_budget.upper_bound[SPECIAL_LEAP_LABELS[0]] ===
    33333,
);

console.log(
  "\n--- enabled survives by label (Shards was disabled in the save)",
);
check(
  "Shards disabled",
  loaded.profiles[0].bound_budgets.enabled["Shards"] === false,
);
check(
  "every other label enabled",
  ALL_MATERIAL_LABELS.filter((l) => l !== "Shards").every(
    (l) => loaded.profiles[0].bound_budgets.enabled[l],
  ),
);

console.log("\n--- rosters / regions / bundles");
check(
  "two rosters",
  Object.keys(loaded.roster_mats_owned).sort().join(",") === "0,1",
);
check(
  "regions kept",
  loaded.all_regions[0] === "nae" && loaded.all_regions[1] === "euc",
);
check("shard bag selection kept", loaded.selected_bundles.nae.Shards === 2000);
check(
  "shard bag price kept, as a Shards bundle",
  prices[bundle_key("Shards", 2000)] === 222,
);
check("shard_infos is gone", !("shard_infos" in loaded));

console.log("\n--- transient fields are not resurrected");
check("no roster-level tier", !("tier" in loaded));
check("no budget_snapshot", !("budget_snapshot" in loaded));
check("no adv_cache", !("adv_cache" in loaded));
check(
  "no worker bundles on profile",
  !("optimizer_worker_bundle" in loaded.profiles[0]),
);
check(
  "used_materials dropped",
  loaded.profiles[0].keyed_upgrades["5,20,true,0"]?.used_materials === null,
);
check(
  "keyed progress kept",
  loaded.profiles[0].keyed_upgrades["5,20,true,0"]?.starting_artisan === 0.42,
);
check(
  "keyed state kept",
  JSON.stringify(loaded.profiles[0].keyed_upgrades["5,20,true,0"]?.state) ===
    "[[0,2]]",
);

console.log("\n=== 2. round trip: to_saved -> from_saved is a fixed point ===");
const once = to_saved(loaded);
const twice = to_saved(from_saved({ ...once, version: STORAGE_VERSION }));
check("to_saved is stable", JSON.stringify(once) === JSON.stringify(twice));

console.log("\n=== 3. export / import ===");
const text = export_config(loaded);
const reimported = import_config(text);
check(
  "import matches export",
  JSON.stringify(to_saved(reimported)) === JSON.stringify(once),
);
check("export carries version", JSON.parse(text).version === STORAGE_VERSION);
let threw = false;
try {
  import_config(JSON.stringify({ version: 999 }));
} catch {
  threw = true;
}
check("future version rejected", threw);

console.log("\n=== 4. empty and corrupt saves fall back to defaults ===");
localStorage.clear();
const fresh = load_roster_config();
check(
  "one default profile",
  fresh.profiles.length === 1 && fresh.profiles[0].char_name === "Newchar",
);
check(
  "fallback prices applied",
  input_column_to_num(fresh.mats_prices.nae, true)[
    bundle_key("Red", MATERIALS["Red"].bundle_sizes[0])
  ] === FALLBACK_PRICES[bundle_key("Red", MATERIALS["Red"].bundle_sizes[0])],
);
localStorage.setItem(STORAGE_KEY, "not even compressed");
check("corrupt save -> defaults", load_roster_config().profiles.length === 1);

console.log(
  "\n=== 5. a material added to Constants gets its fallback, not 0 ===",
);
const new_mat = bundle_key("Lava's Breath", 1);
const trimmed = structuredClone(once);
delete trimmed.mats_prices.nae.values[new_mat];
const grown = from_saved({ ...trimmed, version: STORAGE_VERSION });
check(
  "missing label -> template value",
  input_column_to_num(grown.mats_prices.nae, true)[new_mat] ===
    FALLBACK_PRICES[new_mat],
  String(input_column_to_num(grown.mats_prices.nae, true)[new_mat]),
);

console.log("\n=== 6. save size ===");
const old_json = JSON.stringify(v7);
const new_json = JSON.stringify({ version: STORAGE_VERSION, ...once });
console.log(
  `  json       ${old_json.length} B -> ${new_json.length} B  (${Math.round((1 - new_json.length / old_json.length) * 100)}% smaller)`,
);
console.log(
  `  compressed ${LZString.compressToUTF16(old_json).length} -> ${LZString.compressToUTF16(new_json).length} UTF16 chars`,
);

console.log("\n=== 7. stale `keys` in the old save must be ignored ===");
// Real V7 saves carry a `keys` array that drifted out of order, while `data` stayed
// positional against ALL_LABELS. The migration must trust position, not those keys.
localStorage.clear();
const scrambled = structuredClone(v7);
scrambled.profiles[0].bound_budgets[0].keys = T0.slice().reverse();
localStorage.setItem(
  "HF_CONFIG_V7_COMPRESSED",
  LZString.compressToUTF16(JSON.stringify(scrambled)),
);
const from_scrambled = load_roster_config();
const sb = input_column_to_num(from_scrambled.profiles[0].bound_budgets, true);
check(
  "values follow position, not stale keys",
  T0.every((label, i) => sb[label] === bound0[i]),
  T0.filter((l, i) => sb[l] !== bound0[i])
    .slice(0, 3)
    .join(","),
);

console.log("\n=== 8. load, save, load again ===");
// The write path and the V8 read path are separate code. A save that cannot be read
// back by the same version only shows up on a second load, so do a real round trip
// through localStorage rather than calling to_saved/from_saved in memory.
localStorage.clear();
localStorage.setItem(
  "HF_CONFIG_V7_COMPRESSED",
  LZString.compressToUTF16(JSON.stringify(v7)),
);
const first = load_roster_config();
check(
  "first load consumed the V7 key",
  localStorage.getItem("HF_CONFIG_V7_COMPRESSED") === null,
);

write_state({ roster_config: first });
const written = localStorage.getItem(STORAGE_KEY);
check("the save was written under the current key", written !== null);
check(
  "the save is readable lz-string",
  JSON.parse(LZString.decompressFromUTF16(written)).version === STORAGE_VERSION,
);

const second = load_roster_config();
check(
  "second load has the same profiles",
  second.profiles.map((p) => p.char_name).join(",") ===
    first.profiles.map((p) => p.char_name).join(","),
);
check(
  "second load is identical to the first",
  JSON.stringify(to_saved(second)) === JSON.stringify(to_saved(first)),
);

// and a third, to catch a write that is only unstable after the first generation
write_state({ roster_config: second });
const third = load_roster_config();
check(
  "third load is still identical",
  JSON.stringify(to_saved(third)) === JSON.stringify(to_saved(first)),
);
check(
  "no legacy key was resurrected",
  Object.keys(localStorage).length === 1 ||
    localStorage.getItem("HF_CONFIG_V7_COMPRESSED") === null,
);

done();
