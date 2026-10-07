import { computed, ref } from "vue";
import {
  ALL_MATERIAL_LABELS,
  CONVERTIBLE_MATERIALS,
  NUM_BANDS,
} from "@/Utils/Constants";
import { parse_locale_int } from "@/Utils/InputColumn";
import { ChestKind } from "./LoadStorage";
import {
  assumed_tradability,
  chest_overrides,
  chests_for,
  edits,
  material_overrides,
  shown_slot,
  slots,
} from "./ScanStore";
import ITEMS from "../../../../templates/items.json";
import CHESTS from "../../../../templates/chest.json";

// The manifest: what the scan amounts to, as materials per ownership band plus the select-one
// chests still to be opened. It sits between the scanner and the rest of the site.

const BAND_OF = { CharBound: 0, RosterBound: 1, Tradable: 2 };

type Bag = Record<string, number>; // label -> amount
type Opens = { kind: ChestKind; items: Record<string, number> };

export interface ManifestChest {
  key: string;
  title: string;
  band: number;
  count: number;
  options: Bag[];
}
// a chest that is listed but not passed on
export interface LooseChest {
  title: string;
  count: number;
}

const ICON_LABEL: Record<string, string> = {};
const TITLE_LABEL: Record<string, string> = {};
for (const item of ITEMS as any[]) {
  if (item.label && item.icon) ICON_LABEL[item.icon] = item.label;
  if (item.label && item.title) TITLE_LABEL[item.title] = item.label;
}
const CHEST_ICONS = new Set((CHESTS as any[]).map((chest) => chest.icon));
// what a chest listed inside another chest holds, which no tooltip says
const OPENS: Record<string, Opens> = Object.fromEntries(
  (CHESTS as any[])
    .filter((chest) => chest.opens)
    .map((chest) => [chest.title, chest.opens]),
);

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
function alternatives(title: string, amount: number): Bag[] {
  if (TITLE_LABEL[title]) return [{ [TITLE_LABEL[title]]: amount }];
  const opens = OPENS[title];
  if (!opens || opens.kind === "Random") return [];
  const parts = Object.entries(opens.items).map(([inner, each]) =>
    alternatives(inner, each * amount),
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
  // from obtain-all chests
  const extra: Record<string, number[]> = {};
  for (const label of ALL_MATERIAL_LABELS) {
    owned[label] = [null, null, null];
    extra[label] = [0, 0, 0];
  }
  const grouped = new Map<string, ManifestChest>();
  const random: LooseChest[] = [];
  const unknown: LooseChest[] = [];
  let missing = 0;

  // everything inside a chest takes the chest's band
  function open(
    kind: ChestKind,
    items: [string, number][],
    count: number,
    band: number,
    title: string,
  ) {
    if (kind === "Random") return add_loose(random, title, count);
    if (kind === "SelectOne") {
      const options = items.flatMap(([inner, each]) =>
        alternatives(inner, each),
      );
      if (!options.length) return add_loose(unknown, title, count);
      const key = `${title}|${band}|${JSON.stringify(options)}`;
      const old = grouped.get(key);
      if (old) old.count += count;
      else grouped.set(key, { key, title, band, count, options });
      return;
    }
    for (const [inner, each] of items) {
      const label = TITLE_LABEL[inner];
      const opens = OPENS[inner];
      if (label) extra[label][band] += each * count;
      else if (opens) {
        open(opens.kind, Object.entries(opens.items), each * count, band, inner);
      } else add_loose(unknown, inner, each * count);
    }
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
    const band =
      BAND_OF[
        (edit
          ? (edit.tradability ?? assumed_tradability(address))
          : slot.tradability) ?? ""
      ];
    if (amount == null || band === undefined) {
      // an edit never holds the manifest back; without a band it is only left out
      if (!edit) missing++;
      continue;
    }
    const label = ICON_LABEL[item];
    if (label) {
      owned[label][band] = (owned[label][band] ?? 0) + amount;
      continue;
    }
    if (!CHEST_ICONS.has(item)) continue;
    const found = chests_for(address, item);
    const chest =
      found.find((chest) => digits(chest.amount) === amount) ?? found[0];
    if (!chest) {
      if (edit) add_loose(unknown, item, amount);
      else missing++;
      continue;
    }
    open(
      chest.kind,
      chest.contents.map((content) => [content.item, content.amount]),
      amount,
      band,
      chest.title ?? chest.last_read_title,
    );
  }
  return { owned, extra, chests: [...grouped.values()], random, unknown, missing };
});

// indexed by band; a converted band's T4 materials become Serca ones
export const convert = ref([false, false, false]);
// the character and tier the toggles were defaulted for
export const convert_owner = ref("");

const cell_key = (label: string, band: number) => `${label}|${band}`;

// what the input shows: the override, else "n+k", else nothing (a question mark)
export function cell_text(label: string, band: number): string {
  const override = material_overrides.value[cell_key(label, band)];
  if (override !== undefined) return override;
  const n = scanned.value.owned[label][band];
  const k = scanned.value.extra[label][band];
  if (n === null && !k) return "";
  return (n ?? 0).toLocaleString() + (k ? "+" + k.toLocaleString() : "");
}

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
  for (const label of ALL_MATERIAL_LABELS) {
    materials[label] = bands.map((band) => cell_value(label, band));
    incoming[label] = [0, 0, 0];
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
  const chests = scanned.value.chests.map((chest) => ({
    ...chest,
    count: chest_overrides.value[chest.key] ?? chest.count,
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
  }));
  return { ...scanned.value, materials, incoming, chests };
});
