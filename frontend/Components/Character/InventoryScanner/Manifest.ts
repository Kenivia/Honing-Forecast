import { computed, ref } from "vue";
import {
  ALL_MATERIAL_LABELS,
  CONVERTIBLE_MATERIALS,
  NUM_BANDS,
} from "@/Utils/Constants";
import { parse_locale_int } from "@/Utils/InputColumn";
import { ChestKind } from "./LoadStorage";
import {
  added_chests,
  assumed_tradability,
  chest_overrides,
  chests_for,
  deleted_chests,
  edits,
  material_overrides,
  shown_slot,
  slot_place,
  slots,
} from "./ScanStore";
import ITEMS from "../../../../templates/items.json";
import CHESTS from "../../../../templates/chests.json";

// The manifest: what the scan amounts to, as materials per ownership band plus the select-one
// chests still to be opened. It sits between the scanner and the rest of the site.

const BAND_OF = { CharBound: 0, RosterBound: 1, Tradable: 2 };

type Bag = Record<string, number>; // label -> amount
// one row of templates/chests.json; contents are title, amount, and the chest it is
type Opens = {
  id: number;
  title: string;
  kind: ChestKind;
  contents: [string, number, number | null][];
  // how each content is bound, where one is bound more loosely than to a character
  content_binds?: (string | null)[];
  // only there to tell a slot from the chest it is drawn like
  irrelevant?: boolean;
};

export interface ManifestChest {
  key: string;
  title: string;
  band: number;
  count: number;
  options: Bag[];
}
// a slot the manifest is short of, and why
export interface MissingSlot {
  place: string;
  item: string;
  why: string;
}
// a chest that is listed but not passed on
export interface LooseChest {
  title: string;
  count: number;
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

// What a chest opens to, all the way down, as text: two chests are the same to the manifest when
// this is.
function opens_to(id: number): string {
  const chest = OPENS[id];
  const contents = chest.contents.map(([title, amount, inner]) =>
    inner ? [opens_to(inner), amount] : [title, amount],
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

const digits = (text: string | null) =>
  text ? Number(text.replace(/\D/g, "")) : null;

function merge(bags: Bag[]): Bag {
  const out: Bag = {};
  for (const bag of bags) {
    for (const [label, amount] of Object.entries(bag)) {
      out[label] = (out[label] ?? 0) + amount;
    }
  }
  return out;
}

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

const add_loose = (list: LooseChest[], title: string, count: number) => {
  const old = list.find((chest) => chest.title === title);
  if (old) old.count += count;
  else list.push({ title, count });
};

// Straight off the slot grid, before overrides and conversion. Only slots that are good or were
// set by hand count; the rest are `missing`.
export const scanned = computed(() => {
  const owned: Record<string, (number | null)[]> = {};
  for (const label of ALL_MATERIAL_LABELS) owned[label] = [null, null, null];
  // chests with a choice to make, and chests with none: obtain-all, or one option left
  const grouped = new Map<string, ManifestChest>();
  const sure = new Map<string, ManifestChest>();
  const random: LooseChest[] = [];
  const unknown: LooseChest[] = [];
  const missing: MissingSlot[] = [];

  function group(title: string, band: number, count: number, options: Bag[]) {
    const into = options.length === 1 ? sure : grouped;
    const key = `${title}|${band}|${JSON.stringify(options)}`;
    const old = into.get(key);
    if (old) old.count += count;
    else into.set(key, { key, title, band, count, options });
  }

  // What is inside a chest takes the chest's band, or its own where it is bound more loosely. No
  // select-one of the table holds such a thing, so its options all take the chest's.
  function open(chest: Opens, count: number, band: number) {
    const { kind, title } = chest;
    if (kind === "Random") return add_loose(random, title, count);
    if (kind === "SelectOne") {
      // an option that is no material, or a chest of chance, is not offered
      const options = chest.contents.flatMap(([inner, each, id]) =>
        alternatives(inner, each, id),
      );
      if (options.length) group(title, band, count, options);
      return;
    }
    // an obtain-all inside is part of this chest; any other chest inside is opened in turn
    const bags: Bag[] = [{}, {}, {}];
    const gather = (chest: Opens, times: number, band: number) => {
      chest.contents.forEach(([inner, each, id], at) => {
        const own = Math.max(
          band,
          BAND_OF[chest.content_binds?.[at] ?? ""] ?? 0,
        );
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
      if (Object.keys(bag).length) group(title, own, count, [bag]);
    });
  }

  const keys = new Set([...slots.value.keys(), ...Object.keys(edits.value)]);
  for (const key of keys) {
    const edit = edits.value[key];
    const slot = shown_slot(slots.value.get(key));
    if (!edit && (!slot || slot.status === "Irrelevant")) continue;
    const item = edit ? edit.item : slot.icon_name_score?.[0];
    if (!item) continue;
    // a slot set by hand is as good as a read one: no amount is a single item, and no
    // tradability is what is assumed for its window
    const amount = edit
      ? (edit.amount ?? 1)
      : slot.status === "Good"
        ? slot.value
        : null;
    const address = edit?.address ?? slot.address;
    const miss = () =>
      missing.push({
        place: slot_place(address),
        item: item_name(item),
        why: slot.reason || "Its number is still being read.",
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
      owned[label][band] = (owned[label][band] ?? 0) + amount;
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
      if (edit || read) add_loose(unknown, read?.title ?? item, amount);
      else miss();
      continue;
    }
    open(chest, amount, band);
  }
  return {
    owned,
    chests: [...grouped.values()],
    opened: [...sure.values()],
    random,
    unknown,
    missing,
  };
});

// indexed by band; a converted band's T4 materials become Serca ones
export const convert = ref([false, false, false]);
// the character and tier the toggles were defaulted for
export const convert_owner = ref("");

const cell_key = (label: string, band: number) => `${label}|${band}`;

// what the input shows: the override, else what the slots hold, else nothing (a question mark)
export function cell_text(label: string, band: number): string {
  const override = material_overrides.value[cell_key(label, band)];
  if (override !== undefined) return override;
  return scanned.value.owned[label][band]?.toLocaleString() ?? "";
}

// without the deleted ones, and with the counts the user set
const kept = (chests: ManifestChest[]) =>
  chests
    .filter((chest) => !deleted_chests.value.includes(chest.key))
    .map((chest) => ({
      ...chest,
      count: chest_overrides.value[chest.key] ?? chest.count,
    }));

// an emptied cell goes back to what was scanned
export function set_cell_text(label: string, band: number, text: string) {
  const value = parse_locale_int(text);
  if (isFinite(value)) {
    material_overrides.value[cell_key(label, band)] = value.toLocaleString();
  } else delete material_overrides.value[cell_key(label, band)];
}

const cell_value = (label: string, band: number) =>
  parse_locale_int(cell_text(label, band)) || 0;

const SERCA_OF = Object.fromEntries(
  CONVERTIBLE_MATERIALS.map((mat) => [mat.from.label, mat]),
);

// What the rest of the site takes: overrides applied, then conversion. Converted tradables come
// out roster-bound. Chest options convert without rounding, since a count multiplies them later.
export const manifest = computed(() => {
  const bands = [...Array(NUM_BANDS).keys()];
  const materials: Record<string, number[]> = {};
  // gained from T4, shown beside the input
  const incoming: Record<string, number[]> = {};
  // from the chests with no choice to make, shown beside the input
  const extra: Record<string, number[]> = {};
  for (const label of ALL_MATERIAL_LABELS) {
    incoming[label] = [0, 0, 0];
    extra[label] = [0, 0, 0];
  }
  const opened = kept(scanned.value.opened);
  for (const chest of opened) {
    for (const [label, amount] of Object.entries(chest.options[0])) {
      extra[label][chest.band] += amount * chest.count;
    }
  }
  for (const label of ALL_MATERIAL_LABELS) {
    materials[label] = bands.map(
      (band) => cell_value(label, band) + extra[label][band],
    );
  }
  for (const mat of CONVERTIBLE_MATERIALS) {
    for (const band of bands) {
      if (!convert.value[band]) continue;
      const have = materials[mat.from.label][band];
      materials[mat.from.label][band] = have % mat.from.ratio;
      incoming[mat.label][Math.min(band, 1)] += Math.floor(
        have / mat.from.ratio,
      );
    }
  }
  for (const label of ALL_MATERIAL_LABELS) {
    for (const band of bands) materials[label][band] += incoming[label][band];
  }
  const chests = kept([...scanned.value.chests, ...added_chests.value]).map(
    (chest) => ({
      ...chest,
      options: chest.options.map((option) =>
        merge(
          Object.entries(option).map(([label, amount]) => {
            const mat = convert.value[chest.band] && SERCA_OF[label];
            return mat
              ? { [mat.label]: amount / mat.from.ratio }
              : { [label]: amount };
          }),
        ),
      ),
    }),
  );
  return { ...scanned.value, materials, incoming, extra, chests, opened };
});
