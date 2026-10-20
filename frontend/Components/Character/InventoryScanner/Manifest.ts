import { computed, ref } from "vue";
import {
  ALL_MATERIAL_LABELS,
  NUM_BANDS,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import { column_labels, set_cell } from "@/Utils/InputColumn";
import { add_stack, type Bag, type ManifestChest, merge } from "@/Utils/Chest";
import {
  add_loose,
  add_material,
  CURRENCY_PAGES,
  empty_content,
  other_pages,
  own_pages,
  page_key,
  PageContent,
  ROSTER_WINDOWS,
  roster_unchanged,
  save_pages,
  ScanPage,
  sum_pages,
  take_snapshot,
} from "@/Utils/Scans";
import { useRosterStore } from "@/Stores/RosterConfig";
import { merge_lower_tier } from "@/WasmInterface/PayloadBuilder";
import { ChestKind, SlotAddress } from "./LoadStorage";
import {
  assumed_tradability,
  chests_for,
  currencies,
  edits,
  has_progress,
  roster_cover,
  seen_pages,
  set_edit,
  shown_slot,
  slot_place,
  slots,
  WINDOW_NAMES,
} from "./ScanStore";
import ITEMS from "../../../../scanner_setup/templates/items.json";
import CHESTS from "../../../../scanner_setup/templates/chests.json";

// The manifest: what the scan amounts to, as materials per ownership band plus the chests still
// to be opened, merged with the pages saved by earlier exports. It sits between the scanner and
// the rest of the site.

const BAND_OF = { CharBound: 0, RosterBound: 1, Tradable: 2 };
export const BAND_NAMES = ["Char bound", "Roster bound", "Tradable"];
export const CURRENCY_LABELS = {
  Silver: "Silver",
  Gold: "Gold",
  Shards: "Shards",
};

// one row of scanner_setup/templates/chests.json; contents are title, amount, and the chest it is
type Opens = {
  id: number;
  title: string;
  icon: string;
  kind: ChestKind;
  contents: [string, number, number | null][];
  // how each content is bound, where one is gold or bound more loosely than to a character
  content_binds?: (string | null)[];
  // only there to tell a slot from the chest it is drawn like
  irrelevant?: boolean;
};

// a slot the manifest is short of, and why; a currency has no address
export interface MissingSlot {
  place: string;
  item: string;
  why: string;
  address: SlotAddress | null;
}

const ICON_LABEL: Record<string, string> = {};
// An icon several materials are drawn with (the books) says nothing on its own: the tooltip
// says which, or the user does, picking one of these.
export const SHARED_ICONS: Record<string, string[]> = {};
// what a chest gives as currency, which no slot holds
const TITLE_LABEL: Record<string, string> = { Gold: "Gold", Silver: "Silver" };
for (const item of ITEMS as any[]) {
  if (item.label && item.title) TITLE_LABEL[item.title] = item.label;
  if (!item.label || !item.icon) continue;
  const labels = (ITEMS as any[])
    .filter((other) => other.icon === item.icon && other.label)
    .map((other) => other.label);
  if (new Set(labels).size === 1) ICON_LABEL[item.icon] = item.label;
  else SHARED_ICONS[item.icon] = [...new Set(labels)];
}
const SHARED_LABELS = new Set(Object.values(SHARED_ICONS).flat());
// every chest of the table, by id
const OPENS: Record<number, Opens> = Object.fromEntries(
  (CHESTS as any[]).map((chest) => [chest.id, chest]),
);
// slot icon -> the titles of the chests drawn with it that open to something counted
const CHEST_ICONS: Record<string, string[]> = {};
for (const chest of CHESTS as any[]) {
  if (!chest.top || chest.irrelevant) continue;
  const titles = (CHEST_ICONS[`${chest.icon}@${chest.rarity}`] ??= []);
  if (!titles.includes(chest.title)) titles.push(chest.title);
}

// what to call a slot's icon: a chest's is named after its art, which says nothing
export function item_name(icon: string | null) {
  const titles = icon ? CHEST_ICONS[icon] : undefined;
  if (!titles) return icon;
  const more = titles.length > 2 ? ` and ${titles.length - 2} more` : "";
  return titles.slice(0, 2).join(" / ") + more;
}

// What a chest opens to, all the way down, as text, gold with how it is bound: two chests are the
// same to the manifest when this is.
function opens_to(id: number): string {
  const chest = OPENS[id];
  const contents = chest.contents.map(([title, amount, inner], at) =>
    inner
      ? [opens_to(inner), amount]
      : title === "Gold"
        ? [title, amount, chest.content_binds?.[at]]
        : [title, amount],
  );
  // the order they are listed in says nothing
  return JSON.stringify([
    chest.kind,
    contents.map((x) => JSON.stringify(x)).sort(),
  ]);
}

// The one chest these are, as far as what they open to goes. Chests that list the same things
// can hold different chests inside, and then there is no telling.
function settle(variants: number[]): Opens | undefined {
  const [first, ...rest] = variants;
  if (first === undefined) return undefined;
  const same = rest.every((other) => opens_to(other) === opens_to(first));
  return same ? OPENS[first] : undefined;
}

// where a saved page is, in words
export function page_place(window: string, page: number) {
  if (WINDOW_NAMES[window]) return `${WINDOW_NAMES[window]} page ${page + 1}`;
  return Object.keys(CURRENCY_PAGES)[page];
}

const digits = (text: string | null) =>
  text ? Number(text.replace(/\D/g, "")) : null;

// The alternatives one row of a select-one chest stands for. A chest inside it is flattened: an
// obtain-all is its contents, a select-one is its own options.
function alternatives(
  title: string,
  amount: number,
  chest: number | null,
): Bag[] {
  if (TITLE_LABEL[title]) return [{ [TITLE_LABEL[title]]: amount }];
  const opens = chest ? OPENS[chest] : undefined;
  if (!opens || opens.kind === "Random") return [];
  const parts = opens.contents.map(([inner, each, id]) =>
    alternatives(inner, each * amount, id),
  );
  if (opens.kind === "SelectOne") return parts.flat();
  return [merge(parts.flatMap((part) => (part.length === 1 ? part : [])))];
}

// Whether roster storage is taken from the pages saved by an earlier export, its grid covered and
// whatever is read there ignored. It is unless no roster page was ever saved, or the roster's
// budgets changed since the last snapshot. The user can flip it either way.
export const roster_saved = computed(() => {
  const { roster_config, active_profile } = useRosterStore();
  return roster_config.scan_pages.some(
    (page) =>
      page.window === "Roster" &&
      page.char_id === null &&
      page.roster_id === active_profile.roster_id,
  );
});
export const roster_covered = computed(() => {
  if (!roster_saved.value) return false;
  if (roster_cover.value !== null) return roster_cover.value;
  const { roster_config, active_profile } = useRosterStore();
  return roster_unchanged(roster_config, active_profile.roster_id);
});

// Straight off the slot grid, per page, before conversion. Only slots that are good or were set
// by hand count; the rest are `missing`. A page is `seen` once it was open in game or anything on
// it counts. A currency is a page of its own, its roster-wide bands apart from its char-bound one.
export const live = computed(() => {
  const pages: Record<string, PageContent> = {};
  const stacks: Record<string, Map<string, ManifestChest>> = {};
  const missing: MissingSlot[] = [];
  const seen = new Set<string>();
  const content = (key: string) => (pages[key] ??= empty_content());
  const covered = (window: string) =>
    window === "Roster" && roster_covered.value;

  function group(
    key: string,
    chest: Opens,
    band: number,
    count: number,
    options: Bag[],
    obtain_all: boolean,
  ) {
    const { title, icon } = chest;
    // an obtain-all and a select-one are never one stack
    add_stack(
      (stacks[key] ??= new Map()),
      { title, band, count, options, obtain_all, icons: [icon] },
      `${obtain_all}|`,
    );
  }

  // What is inside a chest takes the chest's band, or its own where it is bound more loosely. No
  // select-one of the table holds such a thing, or gold, so its options all take the chest's.
  function open(key: string, chest: Opens, count: number, band: number) {
    const { kind, title } = chest;
    if (kind === "Random") return add_loose(content(key).random, title, count);
    if (kind === "SelectOne") {
      // an option that is no material, or a chest of chance, is not offered
      const options = chest.contents.flatMap(([inner, each, id]) =>
        alternatives(inner, each, id),
      );
      if (options.length) group(key, chest, band, count, options, false);
      return;
    }
    // an obtain-all inside is part of this chest; any other chest inside is opened in turn
    const bags: Bag[] = [{}, {}, {}];
    const gather = (chest: Opens, times: number, band: number) => {
      chest.contents.forEach(([inner, each, id], at) => {
        const bind = BAND_OF[chest.content_binds?.[at] ?? ""];
        const own = Math.max(band, bind ?? 0);
        const label = TITLE_LABEL[inner];
        const opens = id ? OPENS[id] : undefined;
        if (label) bags[own][label] = (bags[own][label] ?? 0) + each * times;
        else if (opens?.kind === "ObtainAll") gather(opens, each * times, own);
        else if (opens) open(key, opens, each * times * count, own);
      });
    };
    gather(chest, 1, band);
    // listed once for each band it gives into
    bags.forEach((bag, own) => {
      if (Object.keys(bag).length) group(key, chest, own, count, [bag], true);
    });
  }

  for (const key of Object.keys(seen_pages.value)) {
    if (!covered(key.split(" ")[0])) seen.add(key);
  }
  const keys = new Set([...slots.value.keys(), ...Object.keys(edits.value)]);
  for (const key of keys) {
    const edit = edits.value[key];
    const slot = shown_slot(slots.value.get(key));
    if (!edit && (!slot || slot.status === "Irrelevant")) continue;
    const address = edit?.address ?? slot.address;
    if (covered(address.inventory_type)) continue;
    const item = edit ? edit.item : slot.icon_name_score?.[0];
    if (!item) continue;
    const page = page_key(address.inventory_type, address.page_num);
    seen.add(page);
    // a slot set by hand is as good as a read one: no amount is a single item, and no
    // tradability is what is assumed for its window
    const amount = edit
      ? (edit.amount ?? 1)
      : slot.status === "Good"
        ? slot.value
        : null;
    const miss = () =>
      missing.push({
        place: slot_place(address),
        item: item_name(item),
        why: slot.reason || "Its number is still being read.",
        address,
      });
    const band =
      BAND_OF[
        (edit
          ? (edit.tradability ?? assumed_tradability(address))
          : slot.tradability) ?? ""
      ];
    if (amount == null || band === undefined) {
      // an edit never holds the manifest back; without a band it is only left out
      if (!edit) miss();
      continue;
    }
    // a shared icon set by hand is set to the material itself
    const label =
      (edit ? null : slot.label) ??
      ICON_LABEL[item] ??
      (SHARED_LABELS.has(item) ? item : undefined);
    if (label) {
      add_material(content(page), label, band, amount);
      continue;
    }
    if (!CHEST_ICONS[item]) continue;
    // the chest read off a tooltip, else whatever its icon can only be
    const found = chests_for(address, item);
    const read =
      found.find((chest) => digits(chest.amount) === amount) ?? found[0];
    const variants = read?.variants ?? slot?.variants ?? [];
    if (variants.length && variants.every((id) => OPENS[id].irrelevant))
      continue;
    const chest = settle(variants);
    if (!chest) {
      // read, but as chests that hold different chests inside
      if (edit || read)
        add_loose(content(page).unknown, read?.title ?? item, amount);
      else miss();
      continue;
    }
    open(page, chest, amount, band);
  }
  // currencies sit in no slot, and come with their bands
  for (const currency of currencies.value) {
    const label = CURRENCY_LABELS[currency.currency];
    if (currency.status !== "Good" || !currency.bands) {
      missing.push({
        place: "Currencies",
        item: label,
        why: currency.reason,
        address: null,
      });
      continue;
    }
    const at = CURRENCY_PAGES[currency.currency];
    const keys = [page_key("CharCurrency", at), page_key("Currency", at)];
    keys.forEach((key) => seen.add(key));
    currency.bands.forEach((amount, band) =>
      add_material(content(keys[Math.min(band, 1)]), label, band, amount),
    );
  }
  for (const [key, stack] of Object.entries(stacks)) {
    content(key).chests = [...stack.values()];
  }
  return { pages, seen: [...seen], missing };
});

const split_key = (key: string) => {
  const at = key.lastIndexOf(" ");
  return { window: key.slice(0, at), page: Number(key.slice(at + 1)) };
};

// What an export writes: the pages seen now, this character's and roster's saved pages that were
// not, and what the roster's other characters hold that is not bound to them. Nothing is
// converted; chests are passed on whole, with both tiers in them.
export const manifest = computed(() => {
  const { roster_config, active_profile } = useRosterStore();
  const { roster_id, id } = active_profile;
  const { pages, seen, missing } = live.value;
  const read = seen.map((key) => ({
    ...split_key(key),
    content: pages[key] ?? empty_content(),
  }));
  const saved = own_pages(roster_config, roster_id, id).filter(
    (page) => !seen.includes(page_key(page.window, page.page)),
  );
  const others = other_pages(roster_config, roster_id, id);
  const own = sum_pages([...read.map((page) => page.content), ...saved]);
  // what is bound to another character is its own
  const shared = others.map((page) => ({
    ...empty_content(),
    materials: Object.fromEntries(
      Object.entries(page.materials).map(([label, bands]) => [
        label,
        bands.map((amount, band) => (band ? amount : 0)),
      ]),
    ),
    chests: page.chests.filter((chest) => chest.band),
  }));
  const all = sum_pages([own, ...shared]);

  const bands = [...Array(NUM_BANDS).keys()];
  const materials: Record<string, number[]> = {};
  for (const label of ALL_MATERIAL_LABELS) {
    materials[label] = bands.map((band) => all.materials[label]?.[band] ?? 0);
  }
  // what the calculator adds from T4 for a Serca character, shown beside the amount
  const per_band = bands.map((band) =>
    Object.fromEntries(
      ALL_MATERIAL_LABELS.map((label) => [label, materials[label][band]]),
    ),
  );
  const merged = merge_lower_tier(
    active_profile.tier,
    per_band[0],
    per_band[1],
    per_band[2],
    active_profile.bound_budgets.enabled,
  );
  const incoming: Record<string, number[]> = {};
  for (const label of ALL_MATERIAL_LABELS) {
    incoming[label] = [merged.bound, merged.roster, merged.tradable].map(
      (owned, band) => owned[label] - per_band[band][label],
    );
  }
  return {
    materials,
    incoming,
    chests: all.chests,
    random: own.random,
    unknown: own.unknown,
    missing,
    read,
    saved,
    others,
  };
});

const copy = <T>(value: T): T => JSON.parse(JSON.stringify(value));

// Overwrites the budgets and chests of this character and its roster with the manifest, and
// saves the pages seen now. Slots still unresolved are set irrelevant first. The roster's budgets
// are snapshotted before and after.
export function export_manifest() {
  for (const slot of live.value.missing) {
    if (slot.address)
      set_edit({
        address: slot.address,
        item: null,
        amount: null,
        tradability: null,
      });
  }
  // read first: the snapshot can make roster storage covered by default
  const { materials, chests, read } = manifest.value;
  const store = useRosterStore();
  const config = store.roster_config;
  const profile = store.active_profile;
  take_snapshot(config, profile.roster_id, "Before export");

  [
    profile.bound_budgets,
    store.active_roster_mats_owned,
    store.active_tradable_mats_owned,
  ].forEach((column, band) => {
    for (const label of column_labels(column)) {
      if (materials[label])
        set_cell(column, label, String(materials[label][band]));
    }
  });
  // free taps are one number, whatever the band
  for (const label of SPECIAL_LEAP_LABELS) {
    const total = materials[label].reduce((sum, amount) => sum + amount, 0);
    set_cell(profile.special_budget, label, String(total));
  }
  // what any character of the roster can open belongs to the roster
  profile.chests = copy(chests.filter((chest) => !chest.band));
  config.roster_chests[profile.roster_id] = copy(
    chests.filter((chest) => chest.band),
  );

  const at = Date.now();
  save_pages(
    config,
    read.map(
      ({ window, page, content }): ScanPage => ({
        ...copy(content),
        roster_id: profile.roster_id,
        char_id: ROSTER_WINDOWS.includes(window) ? null : profile.id,
        window,
        page,
        at,
        by: profile.id,
      }),
    ),
  );
  take_snapshot(config, profile.roster_id, "After export");
  roster_cover.value = null;
  exported_read.value = JSON.stringify(manifest.value.read);
}

// the pages read as of the last export
const exported_read = ref<string | null>(null);
// whether the scan has anything an export would change
export const scan_unexported = computed(
  () =>
    has_progress.value &&
    JSON.stringify(manifest.value.read) !== exported_read.value,
);
