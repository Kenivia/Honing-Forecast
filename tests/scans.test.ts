import { check, done, equal, section } from "./helpers";
// The manifest merged across pages, exports and characters, and the budget snapshots.
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { new_char_profile } from "@/Stores/CharacterProfile";
import { from_saved } from "@/Stores/SavedConfig";
import { input_column_to_num, set_cell } from "@/Utils/InputColumn";
import { snapshot_diff, snapshot_of, sum_pages } from "@/Utils/Scans";
import {
  currencies,
  edits,
  reset_scan,
  roster_cover,
  seen_pages,
  set_edit,
  slots,
} from "@/Components/Character/InventoryScanner/ScanStore";
import {
  export_manifest,
  manifest,
  roster_covered,
} from "@/Components/Character/InventoryScanner/Manifest";

setActivePinia(createPinia());
const store = useRosterStore();
store.replace_config(from_saved({}));
const config = store.roster_config;
const a = config.profiles[0];
const b = { ...new_char_profile(), char_name: "Bee" };
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
check("nothing saved: roster storage not covered", !roster_covered.value);
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
check("saved and unchanged: covered", roster_covered.value);
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

section("A again, roster budget changed by hand");
as(0);
set_cell(store.active_tradable_mats_owned, "Red", "1");
roster_cover.value = null;
check("changed since the snapshot: not covered", !roster_covered.value);
roster_cover.value = true;
check("the user can cover it anyway", roster_covered.value);
roster_cover.value = null;
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
  roster_cover.value === null && roster_covered.value,
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
  JSON.stringify(snapshot_of(config, 0, "Manual").tradable) ===
    JSON.stringify(last.tradable),
);

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
