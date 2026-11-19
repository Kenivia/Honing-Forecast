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
  empty_content,
  other_pages,
  own_pages,
  page_key,
  PageContent,
  ROSTER_WINDOWS,
  save_currencies,
  save_pages,
  SavedSlot,
  ScanPage,
  sum_pages,
  take_snapshot,
} from "@/Utils/Scans";
import { useRosterStore } from "@/Stores/RosterConfig";
import { merge_lower_tier } from "@/WasmInterface/PayloadBuilder";
import { ChestKind, Currency, SlotAddress } from "./LoadStorage";
import {
  chests_for,
  edits,
  has_progress,
  seen_pages,
  set_edit,
  shown_currency,
  slot_place,
  slots,
  WINDOW_NAMES,
} from "./ScanStore";
import ITEMS from "../../../../scanner_setup/templates/items.json";
import CHESTS from "../../../../scanner_setup/templates/chests.json";

// The manifest: what the scan amounts to, as materials per ownership band plus the chests still
// to be opened, merged with the pages saved slot by slot by earlier exports. It sits between the
// scanner and the rest of the site.

const BAND_OF = { CharBound: 0, RosterBound: 1, Tradable: 2 };
export const BAND_NAMES = ["Char bound", "Roster bound", "Tradable"];
// as the game's bar at the top of the screen has them
export const CURRENCY_ORDER: Currency[] = ["Silver", "Gold", "Shards"];
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
  top?: boolean;
  rarity?: string;
  binds?: string[];
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

// An icon drawn for chests that open to different things says no more than a shared one: the
// user picks one of its groups, as "<icon>#<n>", and the edit keeps the group's chests.
export const CHEST_GROUPS: Record<string, number[][]> = {};
for (const chest of CHESTS as any[]) {
  if (!chest.top || chest.irrelevant) continue;
  const groups = (CHEST_GROUPS[`${chest.icon}@${chest.rarity}`] ??= []);
  const group = groups.find((ids) => same_as(ids[0]) === same_as(chest.id));
  if (group) group.push(chest.id);
  else groups.push([chest.id]);
}
for (const icon in CHEST_GROUPS)
  if (CHEST_GROUPS[icon].length < 2) delete CHEST_GROUPS[icon];

// what can be picked for an icon
export const choices = (icon: string) =>
  CHEST_GROUPS[icon]?.map((_, n) => `${icon}#${n}`) ?? [icon];

// a choice as the item it sets and, for a group, its chests
export function split_choice(choice: string) {
  const [item, n] = choice.split("#");
  return { item, group: n === undefined ? undefined : CHEST_GROUPS[item][+n] };
}

// the choice for an icon read as these chests, the icon itself while they span groups
export function choice_of(icon: string, variants: number[]) {
  const groups = CHEST_GROUPS[icon];
  const ids = variants.filter((id) => OPENS[id] && !OPENS[id].irrelevant);
  const n = groups?.findIndex((group) => ids.every((id) => group.includes(id)));
  return ids.length && n !== undefined && n >= 0 ? `${icon}#${n}` : icon;
}

// how these chests are bound, when all are bound one way; nothing bound to a character is in roster storage
export function bound_alike(variants: number[], window: string) {
  const binds = variants
    .flatMap((id) => OPENS[id]?.binds ?? [])
    .filter((x) => !ROSTER_WINDOWS.includes(window) || x !== "CharBound");
  return binds.length && binds.every((x) => x === binds[0]) ? binds[0] : null;
}

// what to call a slot's icon: a chest's is named after its art, which says nothing
export function item_name(choice: string | null) {
  if (!choice) return choice;
  const { item, group } = split_choice(choice);
  const titles = group ? titles_of(group) : CHEST_ICONS[item];
  if (!titles) return item;
  const more = titles.length > 2 ? ` and ${titles.length - 2} more` : "";
  const name = titles.slice(0, 2).join(" / ") + more;
  // groups of one icon can share titles, and then the first content tells them apart
  const alike = (CHEST_GROUPS[item] ?? []).filter(
    (other) => titles_of(other).join() === titles.join(),
  );
  const [title, amount] = OPENS[group?.[0]]?.contents[0] ?? [];
  return group && alike.length > 1 ? `${name} (${title} x${amount})` : name;
}
const titles_of = (group: number[]) => [
  ...new Set(group.map((id) => OPENS[id].title)),
];

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

// the same, a chest that only holds one obtain-all chest being that one
function same_as(id: number): string {
  const chest = OPENS[id];
  const [only, ...rest] = chest.contents;
  const inner = only?.[2] ? OPENS[only[2]] : undefined;
  return chest.kind === "ObtainAll" &&
    !rest.length &&
    only[1] === 1 &&
    inner?.kind === "ObtainAll"
    ? same_as(inner.id)
    : opens_to(id);
}

// The one chest these are, as far as what they open to goes. Chests that list the same things
// can hold different chests inside, and then there is no telling.
function settle(variants: number[]): Opens | undefined {
  const [first, ...rest] = variants;
  if (first === undefined) return undefined;
  const same = rest.every((other) => same_as(other) === same_as(first));
  return same ? OPENS[first] : undefined;
}

// where a page is, in words
export const page_place = (window: string, page: number) =>
  `${WINDOW_NAMES[window]} page ${page + 1}`;

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

/// the chests of the table still in the game data
const known = (variants: number[]) => variants.filter((id) => OPENS[id]);
const all_irrelevant = (variants: number[]) =>
  variants.length > 0 && variants.every((id) => OPENS[id].irrelevant);

// What saved or live slots of one page add up to, before conversion.
export function content_of(saved: SavedSlot[]): PageContent {
  const content = empty_content();
  const stacks = new Map<string, ManifestChest>();

  function group(
    chest: Opens,
    band: number,
    count: number,
    options: Bag[],
    obtain_all: boolean,
  ) {
    const { title, icon } = chest;
    // an obtain-all and a select-one are never one stack
    add_stack(
      stacks,
      { title, band, count, options, obtain_all, icons: [icon] },
      `${obtain_all}|`,
    );
  }

  // What is inside a chest takes the chest's band, or its own where it is bound more loosely. No
  // select-one of the table holds such a thing, or gold, so its options all take the chest's.
  function open(chest: Opens, count: number, band: number) {
    const { kind, title } = chest;
    if (kind === "Random") return add_loose(content.random, title, count);
    if (kind === "SelectOne") {
      // an option that is no material, or a chest of chance, is not offered
      const options = chest.contents.flatMap(([inner, each, id]) =>
        alternatives(inner, each, id),
      );
      if (options.length) group(chest, band, count, options, false);
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
        else if (opens) open(opens, each * times * count, own);
      });
    };
    gather(chest, 1, band);
    // listed once for each band it gives into
    bags.forEach((bag, own) => {
      if (Object.keys(bag).length) group(chest, own, count, [bag], true);
    });
  }

  for (const slot of saved) {
    const { item, amount } = slot;
    const band = BAND_OF[slot.tradability ?? ""];
    if (!item || band === undefined) continue;
    // a shared icon set by hand is set to the material itself
    const label =
      slot.label ??
      ICON_LABEL[item] ??
      (SHARED_LABELS.has(item) ? item : undefined);
    if (label) {
      add_material(content, label, band, amount);
      continue;
    }
    if (!CHEST_ICONS[item]) continue;
    const variants = known(slot.variants);
    if (all_irrelevant(variants)) continue;
    const chest = settle(variants);
    // read, but as chests that hold different chests inside
    if (!chest)
      add_loose(content.unknown, OPENS[variants[0]]?.title ?? item, amount);
    else open(chest, amount, band);
  }
  content.chests = [...stacks.values()];
  return content;
}

// the chest read off a tooltip for a slot, else none
function chest_read(address: SlotAddress, item: string, amount: number) {
  const read = chests_for(address, item);
  return read.find((chest) => digits(chest.amount) === amount) ?? read[0];
}

// Straight off the slot grid, per page, as an export saves it. Slots that are good or were set by
// hand are kept, and irrelevant ones so that a later scan knows them; the rest are `missing`. A
// page is `seen` once it was open in game or anything on it counts. The currencies are apart.
export const live = computed(() => {
  const pages: Record<string, SavedSlot[]> = {};
  const missing: MissingSlot[] = [];
  const seen = new Set(Object.keys(seen_pages.value));
  // slots in order thanks to the last export
  let assumed = 0;

  const keys = new Set([...slots.value.keys(), ...Object.keys(edits.value)]);
  for (const key of keys) {
    const edit = edits.value[key];
    const slot = slots.value.get(key);
    if (!edit && !slot.icon_name_score) continue;
    const address = edit?.address ?? slot.address;
    const page = page_key(address.inventory_type, address.page_num);
    const keep = (saved: Omit<SavedSlot, "pos" | "icon">) =>
      (pages[page] ??= []).push({
        pos: [address.pos_in_inv[0], address.pos_in_inv[1]],
        icon: edit ? (edit.icon ?? null) : slot.icon_name_score[0],
        ...saved,
      });
    const nothing = { label: null, tradability: null, amount: 0 };
    if (!edit && slot.assumed) assumed++;
    const item = edit ? edit.item : slot.item;
    if (!edit && slot.status === "Irrelevant") {
      const read = item ? chest_read(address, item, slot.value) : undefined;
      keep({ item, ...nothing, variants: read?.variants ?? slot.variants });
      continue;
    }
    if (!item) {
      if (edit) keep({ item, ...nothing, variants: [] });
      continue;
    }
    seen.add(page);
    // a slot set by hand is as good as a read one: no amount is a single item
    const amount = edit
      ? (edit.amount ?? 1)
      : slot.status === "Good"
        ? slot.value
        : null;
    const miss = () =>
      missing.push({
        place: slot_place(address),
        item: item_name(item),
        why: slot.reason || "Its number is still being recognized by OCR.",
        address,
      });
    const tradability = edit ? edit.tradability : slot.tradability;
    if (amount == null || BAND_OF[tradability ?? ""] === undefined) {
      // an edit never holds the manifest back; without a band it is only left out
      if (!edit) miss();
      continue;
    }
    const label = edit ? null : slot.label;
    if (!CHEST_ICONS[item]) {
      keep({ item, label, tradability, amount, variants: [] });
      continue;
    }
    // the chest read off a tooltip, else whatever its icon can only be
    const read = chest_read(address, item, amount);
    const variants = read?.variants ?? edit?.variants ?? slot?.variants ?? [];
    if (!edit && !read && !all_irrelevant(variants) && !settle(variants)) {
      miss();
      continue;
    }
    keep({ item, label, tradability, amount, variants });
  }

  // currencies sit in no slot, come with their bands, and are always read again
  const currencies = empty_content();
  const values: Record<string, number[]> = {};
  for (const name of CURRENCY_ORDER) {
    const currency = shown_currency(name);
    if (!currency) continue;
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
    values[name] = [...currency.bands];
    currency.bands.forEach((amount, band) =>
      add_material(currencies, label, band, amount),
    );
  }
  return { pages, seen: [...seen], missing, currencies, values, assumed };
});

const split_key = (key: string) => {
  const at = key.lastIndexOf(" ");
  return { window: key.slice(0, at), page: Number(key.slice(at + 1)) };
};

// What an export writes: the pages seen now, this character's and roster's saved pages that were
// not (taken as unchanged), the currencies, and what the roster's other characters hold that is
// not bound to them. Nothing is converted; chests are passed on whole, with both tiers in them.
export const manifest = computed(() => {
  const { roster_config, active_profile } = useRosterStore();
  const { roster_id, id } = active_profile;
  const { pages, seen, missing, currencies, assumed } = live.value;
  const read = seen.map((key) => {
    const slots = pages[key] ?? [];
    return { ...split_key(key), slots, content: content_of(slots) };
  });
  const saved = own_pages(roster_config, roster_id, id)
    .filter((page) => !seen.includes(page_key(page.window, page.page)))
    .map(({ window, page, at, by, slots }) => ({
      window,
      page,
      at,
      by,
      content: content_of(slots),
    }));
  const others = other_pages(roster_config, roster_id, id).map((page) =>
    content_of(page.slots),
  );
  const own = sum_pages([
    ...read.map((page) => page.content),
    ...saved.map((page) => page.content),
    currencies,
  ]);
  // what is bound to another character is its own
  const shared = others.map((content) => ({
    ...empty_content(),
    materials: Object.fromEntries(
      Object.entries(content.materials).map(([label, bands]) => [
        label,
        bands.map((amount, band) => (band ? amount : 0)),
      ]),
    ),
    chests: content.chests.filter((chest) => chest.band),
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
    assumed,
  };
});

// What the last export saved of this character's and its roster's slots, for the scanner to take
// on trust. Plain objects: they are posted to the worker.
export function scan_priors() {
  const { roster_config, active_profile } = useRosterStore();
  const { roster_id, id } = active_profile;
  return own_pages(roster_config, roster_id, id).flatMap((page) =>
    page.slots
      .filter((slot) => slot.icon)
      .map((slot) => ({
        address: {
          inventory_type: page.window,
          page_num: page.page,
          pos_in_inv: [slot.pos[0], slot.pos[1]],
        },
        icon: slot.icon,
        item: slot.item,
        label: slot.label,
        tradability: slot.tradability,
        variants: [...slot.variants],
      })),
  );
}

const copy = <T>(value: T): T => JSON.parse(JSON.stringify(value));

// Overwrites the budgets and chests of this character and its roster with the manifest, and
// saves the pages seen now slot by slot. Slots still unresolved are set irrelevant, but not
// saved. The roster's budgets are snapshotted before and after.
export function export_manifest() {
  const { materials, chests, read } = manifest.value;
  const { values } = live.value;
  for (const slot of live.value.missing) {
    if (slot.address)
      set_edit({
        address: slot.address,
        item: null,
        amount: null,
        tradability: null,
      });
  }
  const store = useRosterStore();
  const config = store.roster_config;
  const profile = store.active_profile;
  take_snapshot(config, profile.roster_id, "Before export", profile.id);

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
    read.map(({ window, page, slots }): ScanPage => ({
      roster_id: profile.roster_id,
      char_id: ROSTER_WINDOWS.includes(window) ? null : profile.id,
      window,
      page,
      at,
      by: profile.id,
      slots: copy(slots),
    })),
  );
  save_currencies(config, {
    roster_id: profile.roster_id,
    char_id: profile.id,
    at,
    values: copy(values),
  });
  take_snapshot(config, profile.roster_id, "After export", profile.id);
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
