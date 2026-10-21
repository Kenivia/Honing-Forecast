import { ALL_MATERIAL_LABELS, NUM_BANDS } from "./Constants";
import { InputColumn, input_column_to_num } from "./InputColumn";
import { add_stack, type ManifestChest } from "./Chest";
import type { RosterConfig } from "@/Stores/RosterConfig";

// Scanned pages as last exported, merged across characters, and snapshots of the budgets.

// a chest that is listed but not passed on
export interface LooseChest {
  title: string;
  count: number;
}

// what one page holds: label -> amount per band, non-zero only
export interface PageContent {
  materials: Record<string, number[]>;
  chests: ManifestChest[];
  random: LooseChest[];
  unknown: LooseChest[];
}

// One page of a window as last exported. Roster storage and the roster-wide part of the
// currencies belong to the roster (`char_id` null), everything else to a character. A currency
// is a page of its own, numbered as `CURRENCY_PAGES`.
export interface ScanPage extends PageContent {
  roster_id: number;
  char_id: string | null;
  window: string;
  page: number;
  at: number;
  // the character it was scanned on
  by: string;
}

export const CURRENCY_PAGES = { Silver: 0, Gold: 1, Shards: 2 };
// windows that belong to the roster; the rest are a character's
export const ROSTER_WINDOWS = ["Roster", "Currency"];

export const page_key = (window: string, page: number) => `${window} ${page}`;

export const empty_content = (): PageContent => ({
  materials: {},
  chests: [],
  random: [],
  unknown: [],
});

export function add_loose(list: LooseChest[], title: string, count: number) {
  const old = list.find((chest) => chest.title === title);
  if (old) old.count += count;
  else list.push({ title, count });
}

export function add_material(
  content: PageContent,
  label: string,
  band: number,
  amount: number,
) {
  if (!amount) return;
  const bands = (content.materials[label] ??= Array(NUM_BANDS).fill(0));
  bands[band] += amount;
}

// several pages as one: materials summed, chests that give the same stacked, in an order that
// does not depend on the pages'
export function sum_pages(pages: PageContent[]): PageContent {
  const out = empty_content();
  const stacks = new Map<string, ManifestChest>();
  const chests = pages
    .flatMap((page) => page.chests)
    .sort((a, b) =>
      a.key === b.key
        ? a.title.localeCompare(b.title)
        : a.key.localeCompare(b.key),
    );
  for (const chest of chests) {
    add_stack(stacks, { ...chest }, `${chest.obtain_all}|`);
  }
  for (const page of pages) {
    for (const [label, bands] of Object.entries(page.materials)) {
      bands.forEach((amount, band) => add_material(out, label, band, amount));
    }
    for (const chest of page.random)
      add_loose(out.random, chest.title, chest.count);
    for (const chest of page.unknown)
      add_loose(out.unknown, chest.title, chest.count);
  }
  out.chests = [...stacks.values()];
  return out;
}

const same_page = (a: ScanPage, b: ScanPage) =>
  a.roster_id === b.roster_id &&
  a.char_id === b.char_id &&
  a.window === b.window &&
  a.page === b.page;

// the saved pages of this roster's own windows and of this character's
export const own_pages = (
  config: RosterConfig,
  roster_id: number,
  char_id: string,
) =>
  config.scan_pages.filter(
    (page) =>
      page.char_id === char_id ||
      (page.char_id === null && page.roster_id === roster_id),
  );

// the saved pages of the roster's other characters
export function other_pages(
  config: RosterConfig,
  roster_id: number,
  char_id: string,
) {
  const others = config.profiles
    .filter(
      (profile) => profile.roster_id === roster_id && profile.id !== char_id,
    )
    .map((profile) => profile.id);
  return config.scan_pages.filter((page) => others.includes(page.char_id));
}

export function save_pages(config: RosterConfig, pages: ScanPage[]) {
  config.scan_pages = [
    ...config.scan_pages.filter(
      (old) => !pages.some((page) => same_page(old, page)),
    ),
    ...pages,
  ];
}

// ============================================================================
// Snapshots
// ============================================================================

// what an optimizer run of a snapshot gave one character, under one payload
export interface SnapshotResult {
  // what the chests give opened as the optimizer chose, by label and band
  gains: Record<string, number[]>;
  // average tradable gold spent, and what the leftovers are worth
  gold: number;
  leftover: number;
  // now and the weeks ahead only: the histogram's chance_within
  chance_within?: Record<string, number[]>;
}

export interface CharSnapshot {
  name: string;
  bound: Record<string, number>;
  special: Record<string, number>;
  chests: ManifestChest[];
  // by payload hash, the latest few only
  results?: Record<string, SnapshotResult>;
}

// The budgets of a whole roster at one time, and the prices of its region then.
export interface Snapshot {
  at: number;
  kind: "Before export" | "After export" | "Manual";
  roster_id: number;
  roster_owned: Record<string, number>;
  tradable: Record<string, number>;
  roster_chests: ManifestChest[];
  chars: Record<string, CharSnapshot>;
  // per bundle key, and the bundle bought per material
  prices: Record<string, number>;
  bundles: Record<string, number>;
  // left out of the income estimate, or starting a new line in it
  ignored?: boolean;
  breaks?: boolean;
}

const nonzero = (column: InputColumn) =>
  Object.fromEntries(
    Object.entries(input_column_to_num(column, true)).filter(
      ([, amount]) => amount,
    ),
  );

const copy = <T>(value: T): T => JSON.parse(JSON.stringify(value));

export function snapshot_of(
  config: RosterConfig,
  roster_id: number,
  kind: Snapshot["kind"],
): Snapshot {
  const chars: Record<string, CharSnapshot> = {};
  for (const profile of config.profiles) {
    if (profile.roster_id !== roster_id) continue;
    chars[profile.id] = {
      name: profile.char_name,
      bound: nonzero(profile.bound_budgets),
      special: nonzero(profile.special_budget),
      chests: copy(profile.chests),
    };
  }
  const region = config.all_regions[roster_id];
  return {
    at: Date.now(),
    kind,
    roster_id,
    roster_owned: nonzero(config.roster_mats_owned[roster_id]),
    tradable: nonzero(config.tradable_mats_owned[roster_id]),
    roster_chests: copy(config.roster_chests[roster_id] ?? []),
    chars,
    prices: nonzero(config.mats_prices[region]),
    bundles: copy(config.selected_bundles[region]),
  };
}

export const latest_snapshot = (config: RosterConfig, roster_id: number) =>
  config.snapshots.findLast((snapshot) => snapshot.roster_id === roster_id);

// the materials and chests alone
export const budgets_of = (snapshot: Snapshot) =>
  JSON.stringify([
    snapshot.roster_owned,
    snapshot.tradable,
    snapshot.roster_chests,
    Object.entries(snapshot.chars).map(
      ([id, { results: _results, ...char }]) => [id, char],
    ),
  ]);

// added unless it is the same as the roster's latest
export function take_snapshot(
  config: RosterConfig,
  roster_id: number,
  kind: Snapshot["kind"],
) {
  const snapshot = snapshot_of(config, roster_id, kind);
  const latest = latest_snapshot(config, roster_id);
  if (!latest || budgets_of(latest) !== budgets_of(snapshot))
    config.snapshots.push(snapshot);
}

// whether the roster and tradable budgets are as the latest snapshot has them
export function roster_unchanged(config: RosterConfig, roster_id: number) {
  const latest = latest_snapshot(config, roster_id);
  if (!latest) return false;
  return (
    JSON.stringify([latest.roster_owned, latest.tradable]) ===
    JSON.stringify([
      nonzero(config.roster_mats_owned[roster_id]),
      nonzero(config.tradable_mats_owned[roster_id]),
    ])
  );
}

// ============================================================================
// Diff
// ============================================================================

export const DIFF_COLUMNS = [
  "Char bound",
  "Roster bound",
  "Tradable",
  "Free taps",
];

export interface ChestChange {
  title: string;
  band: number;
  from: number;
  to: number;
}

function sum_records(records: Record<string, number>[]) {
  const out: Record<string, number> = {};
  for (const record of records) {
    for (const [label, amount] of Object.entries(record))
      out[label] = (out[label] ?? 0) + amount;
  }
  return out;
}

// `char_id` null takes every character of the roster
function side(snapshot: Snapshot, char_id: string | null) {
  const chars = Object.entries(snapshot.chars)
    .filter(([id]) => char_id === null || id === char_id)
    .map(([, char]) => char);
  return {
    columns: [
      sum_records(chars.map((char) => char.bound)),
      snapshot.roster_owned,
      snapshot.tradable,
      sum_records(chars.map((char) => char.special)),
    ],
    chests: [
      ...snapshot.roster_chests,
      ...chars.flatMap((char) => char.chests),
    ],
  };
}

// what changed from one snapshot to the other: per label one change per column, and the chests
// whose counts changed
export function snapshot_diff(
  from: Snapshot,
  to: Snapshot,
  char_id: string | null,
) {
  const [a, b] = [side(from, char_id), side(to, char_id)];
  const rows = ALL_MATERIAL_LABELS.map((label) => ({
    label,
    changes: b.columns.map(
      (column, at) => (column[label] ?? 0) - (a.columns[at][label] ?? 0),
    ),
  })).filter((row) => row.changes.some((change) => change));

  const chests = new Map<string, ChestChange>();
  const count = (list: ManifestChest[], field: "from" | "to") => {
    for (const chest of list) {
      const change = (chests.get(chest.key) ?? {
        title: chest.title,
        band: chest.band,
        from: 0,
        to: 0,
      }) as ChestChange;
      change[field] += chest.count;
      chests.set(chest.key, change);
    }
  };
  count(a.chests, "from");
  count(b.chests, "to");
  return {
    rows,
    chests: [...chests.values()].filter((chest) => chest.from !== chest.to),
  };
}
