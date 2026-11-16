import { check, done, equal, section } from "./helpers";
// The manifest merged across pages, exports and characters, and the budget snapshots.
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { new_char_profile } from "@/Stores/CharacterProfile";
import { from_saved } from "@/Stores/SavedConfig";
import { input_column_to_num, set_cell } from "@/Utils/InputColumn";
import {
  snapshot_diff,
  snapshot_of,
  sum_pages,
  take_snapshot,
} from "@/Utils/Scans";
import {
  currencies,
  edits,
  reset_scan,
  seen_pages,
  set_edit,
  slots,
} from "@/Components/Character/InventoryScanner/ScanStore";
import {
  export_manifest,
  manifest,
  scan_priors,
} from "@/Components/Character/InventoryScanner/Manifest";
import { build_materials } from "@/WasmInterface/PayloadBuilder";

setActivePinia(createPinia());
const store = useRosterStore();
store.replace_config(from_saved({}));
const config = store.roster_config;
const a = config.profiles[0];
const b = { ...new_char_profile(), char_name: "Bee", roster_id: a.roster_id };
store.add_profile(b);
const ids = config.profiles.map((profile) => profile.id);

const BANDS = { CharBound: 0, RosterBound: 1, Tradable: 2 };
function put(
  window: string,
  page: number,
  cell: number,
  item: string,
  amount: number,
  band: keyof typeof BANDS,
) {
  set_edit({
    address: {
      inventory_type: window,
      page_num: page,
      pos_in_inv: [0, cell],
    } as any,
    item,
    amount,
    tradability: band,
  });
}
const see = (...keys: string[]) =>
  keys.forEach((key) => (seen_pages.value[key] = true));
const owned = () => ({
  bound: input_column_to_num(store.active_profile.bound_budgets),
  roster: input_column_to_num(store.active_roster_mats_owned),
  tradable: input_column_to_num(store.active_tradable_mats_owned),
});
const as = (index: number) => {
  reset_scan();
  store.switch_profile(index);
};

section("first export, character A");
as(0);
see("Roster 0", "CharStorage 0", "CharInventory 0");
put("Roster", 0, 0, "Red", 100, "Tradable");
put("CharStorage", 0, 0, "Blue", 50, "CharBound");
put("CharInventory", 0, 0, "Blue", 20, "RosterBound");
currencies.value = [
  {
    currency: "Silver",
    status: "Good",
    reason: "",
    value: 1000,
    bands: [0, 1000, 0],
  },
  {
    currency: "Gold",
    status: "Good",
    reason: "",
    value: 30,
    bands: [10, 0, 20],
  },
];
// read but not resolved: set irrelevant by the export
slots.value = new Map([
  [
    "CharInventory 0 1,1",
    {
      address: {
        inventory_type: "CharInventory",
        page_num: 0,
        pos_in_inv: [1, 1],
      },
      icon_name_score: ["Blue", 0.99],
      alternatives: [],
      amount: "9999",
      tooltip_amount: null,
      tradability: null,
      label: null,
      item: "Blue",
      status: "NeedHover",
      assumed: false,
      reading: false,
      reason: "needs a hover",
      value: 9999,
      variants: [],
    } as any,
  ],
]);
equal("one slot missing", manifest.value.missing.length, 1);
export_manifest();
let now = owned();
equal("A bound", [now.bound.Blue, now.bound.Gold], [50, 10]);
equal("roster", [now.roster.Blue, now.roster.Silver], [20, 1000]);
equal("tradable", [now.tradable.Red, now.tradable.Gold], [100, 20]);
check(
  "unresolved slot set irrelevant",
  edits.value["CharInventory 0 1,1"]?.item === null,
);
equal(
  "two snapshots",
  config.snapshots.map((s) => s.kind),
  ["Before export", "After export"],
);
equal(
  "pages saved with their owners",
  config.scan_pages
    .map(
      (page) =>
        `${page.window} ${page.page} ${page.char_id === null ? "roster" : page.char_id === ids[0] ? "A" : "?"}`,
    )
    .sort(),
  ["CharInventory 0 A", "CharStorage 0 A", "Roster 0 roster"],
);
equal(
  "the unresolved slot is not saved",
  config.scan_pages
    .find((page) => page.window === "CharInventory")
    .slots.map((slot) => slot.pos),
  [[0, 0]],
);
equal(
  "currencies saved apart",
  config.scan_currencies.map((x) => [x.char_id === ids[0], x.values]),
  [[true, { Silver: [0, 1000, 0], Gold: [10, 0, 20] }]],
);

section("character B opens roster storage again");
as(1);
see("Roster 0", "CharInventory 0");
put("Roster", 0, 0, "Red", 999, "Tradable");
put("CharInventory", 0, 0, "Blue", 7, "RosterBound");
put("CharInventory", 0, 1, "Red", 5, "Tradable");
put("CharInventory", 0, 2, "Blue", 3, "CharBound");
currencies.value = [];
export_manifest();
now = owned();
equal("B bound", now.bound.Blue, 3);
equal("roster adds A's and B's", now.roster.Blue, 27);
equal("tradable: roster storage as B read it", now.tradable.Red, 1004);
equal(
  "currencies are never taken from an export: not read, 0",
  [now.roster.Silver, now.tradable.Gold, now.bound.Gold],
  [0, 0, 0],
);
equal("A's bound untouched", input_column_to_num(a.bound_budgets).Blue, 50);
check(
  "B's char pages are B's",
  config.scan_pages.some(
    (page) => page.char_id === ids[1] && page.window === "CharInventory",
  ),
);
equal(
  "roster storage saved again, by B",
  config.scan_pages
    .filter((page) => page.window === "Roster")
    .map((page) => page.by === ids[1]),
  [true],
);

section("a page not opened is taken as unchanged");
as(1);
see("Roster 1");
put("Roster", 1, 0, "Red", 30, "Tradable");
equal(
  "saved pages in use",
  manifest.value.saved.map((page) => `${page.window} ${page.page}`).sort(),
  ["CharInventory 0", "Roster 0"],
);
equal(
  "saved page 1, new page 2, B's inventory",
  manifest.value.materials.Red[2],
  1034,
);
reset_scan();

section("A again, only roster page 2 opened");
as(0);
see("Roster 1");
put("Roster", 1, 0, "Red", 30, "Tradable");
equal(
  "saved pages in use",
  manifest.value.saved.map((page) => `${page.window} ${page.page}`).sort(),
  ["CharInventory 0", "CharStorage 0", "Roster 0"],
);
export_manifest();
now = owned();
equal("tradable: saved page 1, new page 2, B's", now.tradable.Red, 1034);
equal("A bound from its saved pages", now.bound.Blue, 50);

section("snapshots");
const first = config.snapshots[0];
const last = config.snapshots[config.snapshots.length - 1];
const diff = snapshot_diff(first, last, null);
const row = (label: string) =>
  diff.rows.find((r) => r.label === label)?.changes;
equal("Blue: A 50 + B 3 bound, 27 roster", row("Blue"), [53, 27, 0, 0]);
equal("Red tradable", row("Red"), [0, 0, 1034, 0]);
equal(
  "B alone",
  snapshot_diff(first, last, ids[1]).rows.find((r) => r.label === "Blue")
    ?.changes,
  [3, 27, 0, 0],
);
const count = config.snapshots.length;
store.switch_profile(0);
// unchanged budgets are not snapshotted twice
export_manifest();
equal(
  "an export changing nothing adds no snapshot",
  config.snapshots.length,
  count,
);
check(
  "now matches the last snapshot",
  JSON.stringify(
    snapshot_of(config, config.profiles[0].roster_id, "Manual").tradable,
  ) === JSON.stringify(last.tradable),
);
equal("grids snapshotted", last.chars[ids[0]].normal_grid, a.normal_grid);
equal("adv grid snapshotted", last.chars[ids[0]].adv_grid, a.adv_grid);
equal("tier snapshotted", last.chars[ids[0]].tier, a.tier);
equal(
  "an export snapshot names its exporter",
  last.by,
  store.active_profile.id,
);
check(
  "a manual one names none",
  !("by" in snapshot_of(config, a.roster_id, "Manual")),
);
equal("default disabled snapshotted", last.chars[ids[0]].disabled, ["Shards"]);
a.bound_budgets.enabled["Blue"] = false;
const off = snapshot_of(config, a.roster_id, "Manual").chars[ids[0]];
equal("disabled snapshotted", [...off.disabled].sort(), ["Blue", "Shards"]);
equal(
  "a disabled cell keeps its typed value",
  off.bound["Blue"],
  input_column_to_num(a.bound_budgets, true)["Blue"],
);
take_snapshot(config, a.roster_id, "Manual");
equal("disabling alone adds no snapshot", config.snapshots.length, count);
const status = a.normal_grid[0][0];
a.normal_grid[0][0] = status === 0 ? 1 : 0;
take_snapshot(config, a.roster_id, "Manual");
equal("a grid change alone adds no snapshot", config.snapshots.length, count);
a.normal_grid[0][0] = status;
a.bound_budgets.enabled["Blue"] = true;
a.bound_budgets.enabled["Shards"] = true;
check(
  "nothing disabled: no flag",
  !("disabled" in snapshot_of(config, a.roster_id, "Manual").chars[ids[0]]),
);
a.bound_budgets.enabled["Shards"] = false;

section("slots saved for the next scan");
as(0);
see("CharStorage 1");
const at = (col: number) => ({
  inventory_type: "CharStorage",
  page_num: 1,
  pos_in_inv: [0, col],
});
const read = (col: number, more: object) =>
  [
    `CharStorage 1 0,${col}`,
    {
      address: at(col),
      icon_name_score: ["Blue", 0.99],
      alternatives: [],
      amount: "5",
      tooltip_amount: null,
      tradability: "CharBound",
      label: null,
      item: "Blue",
      status: "Good",
      assumed: false,
      reading: false,
      reason: "",
      value: 5,
      variants: [],
      ...more,
    } as any,
  ] as const;
slots.value = new Map([
  read(0, { assumed: true }),
  read(1, {}),
  // set irrelevant by hand at an earlier export
  read(2, { item: null, tradability: null, status: "Irrelevant" }),
]);
equal(
  "a slot put in order by the last export is counted",
  manifest.value.assumed,
  1,
);
set_edit({
  address: at(1) as any,
  item: "Red",
  amount: 2,
  tradability: "Tradable",
});
equal(
  "an edit keeps the icon matched there",
  edits.value["CharStorage 1 0,1"].icon,
  "Blue",
);
export_manifest();
const priors = scan_priors().filter(
  (x) => x.address.inventory_type === "CharStorage" && x.address.page_num === 1,
);
equal(
  "saved as priors, a hand edit as what it was set to",
  priors.map((x) => [x.address.pos_in_inv, x.icon, x.item, x.tradability]),
  [
    [[0, 0], "Blue", "Blue", "CharBound"],
    [[0, 1], "Blue", "Red", "Tradable"],
    [[0, 2], "Blue", null, null],
  ],
);
check(
  "slots set by hand where no icon was matched give no prior",
  scan_priors().every((x) => x.icon),
);
equal("A bound counts the read slot", owned().bound.Blue, 55);
reset_scan();

section("a Serca character's T4 is exported unconverted");
as(0);
a.tier = 1;
see("CharStorage 0", "CharInventory 0");
put("CharStorage", 0, 0, "Fusion", 12, "CharBound");
put("CharInventory", 0, 0, "Fusion", 3, "RosterBound");
put("CharInventory", 0, 1, "Fusion", 4, "Tradable");
equal(
  "shown as what it adds on the calculator",
  manifest.value.incoming["Serca Fusion"],
  [2, 1, 0],
);
export_manifest();
now = owned();
equal(
  "T4 under its own labels",
  [now.bound.Fusion, now.roster.Fusion, now.tradable.Fusion],
  [12, 3, 4],
);
equal(
  "nothing written to Serca",
  [now.bound["Serca Fusion"], now.roster["Serca Fusion"]],
  [0, 0],
);
equal(
  "the payload merges it",
  [
    build_materials()["Serca Fusion"].bound,
    build_materials()["Serca Fusion"].roster,
  ],
  [2, 1],
);
a.tier = 0;

section("stacking across pages");
const chest = (band: number, count: number) => ({
  key: "",
  title: `Box ${band}`,
  band,
  count,
  options: [{ Red: 10 }, { Blue: 20 }],
  obtain_all: false,
});
const merged = sum_pages([
  {
    materials: { Red: [1, 2, 3] },
    chests: [chest(1, 2)],
    random: [],
    unknown: [],
  },
  {
    materials: { Red: [1, 0, 0] },
    chests: [chest(1, 3), chest(0, 1)],
    random: [],
    unknown: [],
  },
]);
const reversed = sum_pages([
  {
    materials: {},
    chests: [chest(1, 3), chest(0, 1)],
    random: [],
    unknown: [],
  },
  { materials: {}, chests: [chest(1, 2)], random: [], unknown: [] },
]);
equal("materials summed", merged.materials.Red, [2, 2, 3]);
equal(
  "chests stacked per band, whatever the order of the pages",
  [merged, reversed].map((x) => x.chests.map((c) => [c.band, c.count])),
  [
    [
      [0, 1],
      [1, 5],
    ],
    [
      [0, 1],
      [1, 5],
    ],
  ],
);

done();
