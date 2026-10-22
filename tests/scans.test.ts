import { check, done, equal, section } from "./helpers";
// The manifest merged across pages, exports and characters, and the budget snapshots.
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { new_char_profile } from "@/Stores/CharacterProfile";
import { from_saved } from "@/Stores/SavedConfig";
import { input_column_to_num, set_cell } from "@/Utils/InputColumn";
import {
  roster_unchanged,
  snapshot_diff,
  snapshot_of,
  sum_pages,
} from "@/Utils/Scans";
import {
  currencies,
  edits,
  reset_scan,
  window_cover,
  seen_pages,
  set_edit,
  slots,
} from "@/Components/Character/InventoryScanner/ScanStore";
import {
  export_manifest,
  manifest,
  covered,
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
check("nothing saved: roster storage not covered", !covered.value.Roster);
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
      status: "NeedHover",
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
  [
    "CharCurrency 0 A",
    "CharCurrency 1 A",
    "CharInventory 0 A",
    "CharStorage 0 A",
    "Currency 0 roster",
    "Currency 1 roster",
    "Roster 0 roster",
  ],
);

section("character B, roster storage covered");
as(1);
check("saved and unchanged: covered", covered.value.Roster);
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
equal(
  "tradable: saved roster storage, not the covered read",
  now.tradable.Red,
  105,
);
equal(
  "currencies kept from A's export",
  [now.roster.Silver, now.tradable.Gold, now.bound.Gold],
  [1000, 20, 0],
);
equal("A's bound untouched", input_column_to_num(a.bound_budgets).Blue, 50);
check(
  "B's char pages are B's",
  config.scan_pages.some(
    (page) => page.char_id === ids[1] && page.window === "CharInventory",
  ),
);
check(
  "roster storage not saved again",
  config.scan_pages.filter((page) => page.window === "Roster").length === 1,
);

section("a roster page open in game that was never saved");
as(1);
const roster_read = () =>
  manifest.value.read
    .filter((page) => page.window === "Roster")
    .map((page) => page.page);
check("only page 1 saved, none open: covered", covered.value.Roster);
see("Roster 0");
check("the saved page open: still covered", covered.value.Roster);
see("Roster 1");
check("page 2 open, never saved: not covered", !covered.value.Roster);
put("Roster", 1, 0, "Red", 30, "Tradable");
equal("both open pages read", roster_read(), [0, 1]);
window_cover.value.Roster = true;
check("the user can still cover it", covered.value.Roster);
equal("covered, nothing read there counts", roster_read(), []);
window_cover.value.Roster = false;
check("and uncover it", !covered.value.Roster);
window_cover.value.Roster = null;
reset_scan();
check("a wipe forgets the page was open", covered.value.Roster);
set_cell(store.active_tradable_mats_owned, "Red", "1");
check("changed budgets still uncover it", !covered.value.Roster);
set_cell(store.active_tradable_mats_owned, "Red", "105");
check(
  "budgets back as snapshotted: covered",
  roster_unchanged(config, config.profiles[0].roster_id) && covered.value.Roster,
);

section("A again, roster budget changed by hand");
as(0);
set_cell(store.active_tradable_mats_owned, "Red", "1");
window_cover.value.Roster = null;
check("changed since the snapshot: not covered", !covered.value.Roster);
window_cover.value.Roster = true;
check("the user can cover it anyway", covered.value.Roster);
window_cover.value.Roster = null;
// only roster page 2 is opened: page 1 and A's own windows come from the last export
see("Roster 1");
put("Roster", 1, 0, "Red", 30, "Tradable");
equal(
  "saved pages in use",
  manifest.value.saved.map((page) => `${page.window} ${page.page}`).sort(),
  [
    "CharCurrency 0",
    "CharCurrency 1",
    "CharInventory 0",
    "CharStorage 0",
    "Currency 0",
    "Currency 1",
    "Roster 0",
  ],
);
export_manifest();
now = owned();
equal("tradable: saved page 1, new page 2, B's", now.tradable.Red, 135);
equal("A bound from its saved pages", now.bound.Blue, 50);
check(
  "after the export the cover is back to the default",
  window_cover.value.Roster === null && covered.value.Roster,
);

section("snapshots");
const first = config.snapshots[0];
const last = config.snapshots[config.snapshots.length - 1];
const diff = snapshot_diff(first, last, null);
const row = (label: string) =>
  diff.rows.find((r) => r.label === label)?.changes;
equal("Blue: A 50 + B 3 bound, 27 roster", row("Blue"), [53, 27, 0, 0]);
equal("Red tradable", row("Red"), [0, 0, 135, 0]);
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
  JSON.stringify(snapshot_of(config, config.profiles[0].roster_id, "Manual").tradable) ===
    JSON.stringify(last.tradable),
);

section("character storage covered");
as(0);
const storage_read = () =>
  manifest.value.read.filter((page) => page.window === "CharStorage").length;
check("A unchanged since its export: covered", covered.value.CharStorage);
put("CharStorage", 0, 0, "Blue", 999, "CharBound");
equal("covered, nothing read there counts", storage_read(), 0);
see("CharStorage 1");
check("a page never saved open: not covered", !covered.value.CharStorage);
reset_scan();
set_cell(store.active_profile.bound_budgets, "Blue", "1");
check("its bound budget changed: not covered", !covered.value.CharStorage);
set_cell(store.active_profile.bound_budgets, "Blue", "50");
check("back as snapshotted: covered", covered.value.CharStorage);
set_cell(store.active_tradable_mats_owned, "Red", "1");
check("the roster's budgets changed: not covered", !covered.value.CharStorage);
set_cell(store.active_tradable_mats_owned, "Red", "135");
window_cover.value.CharStorage = false;
check("the user can uncover it", !covered.value.CharStorage);
as(1);
window_cover.value.CharStorage = null;
check("B never exported its storage: not covered", !covered.value.CharStorage);

section("a Serca character's T4 is exported unconverted");
as(0);
window_cover.value.CharStorage = false;
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
