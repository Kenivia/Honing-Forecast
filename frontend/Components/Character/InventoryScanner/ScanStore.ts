import { computed, ref, shallowRef } from "vue";
import { useThrottleFn } from "@vueuse/core";
import {
  Chest,
  Currency,
  CurrencyResult,
  OneIconConfig,
  ScanResult,
  SlotAddress,
  SlotResult,
} from "./LoadStorage";
import { page_key } from "@/Utils/Scans";

// What the slot grid shows. Module state: it outlives the page and the scanner worker, so the
// grid stays up and editable after capture stops. Only a new capture wipes it.

export interface SlotEdit {
  address: SlotAddress;
  item: string | null;
  amount: number | null;
  tradability: string | null;
  retry?: boolean;
  // the icon matched there when it was set, which a later scan's prior goes by
  icon?: string | null;
  // for a chest, the chests it was set or read as
  variants?: number[];
}

export const WINDOW_NAMES = {
  Roster: "Roster storage",
  CharStorage: "Character storage",
  CharInventory: "Inventory",
};
// where a slot is, in words
export const slot_place = (address: SlotAddress) =>
  `${WINDOW_NAMES[address.inventory_type]} page ${address.page_num + 1}, row ${address.pos_in_inv[0] + 1}, column ${address.pos_in_inv[1] + 1}`;

export const slot_key = (address: SlotAddress) =>
  `${address.inventory_type} ${address.page_num} ${address.pos_in_inv}`;

export const slots = shallowRef(new Map<string, SlotResult>());
export const chests = shallowRef<Chest[]>([]);
export const currencies = shallowRef<CurrencyResult[]>([]);
// the page each located window shows in game
export const game_pages = ref<Record<string, number>>({});
// every page that was open in game during this capture, by `page_key`
export const seen_pages = ref<Record<string, true>>({});
// what the user typed in; the scanner leaves these slots alone
export const edits = ref<Record<string, SlotEdit>>({});
// currencies set by hand, as char bound, roster bound, tradable; the scanner is not told
export const currency_edits = ref<Partial<Record<Currency, number[]>>>({});
// images only arrive when they change, and the same objects are reused so canvases are not redrawn
export const slot_images = new Map<string, OneIconConfig>();

// The chest read in this very slot, else every chest with one of the slot's icons read in its
// column: a pushed-up tooltip only tells the column.
export function chests_for(address: SlotAddress, icon: string | null) {
  if (!icon) return [];
  const icons = [
    icon,
    ...(slots.value.get(slot_key(address))?.alternatives ?? []),
  ];
  const key = slot_key(address);
  const exact = chests.value.filter(
    (chest) => chest.slot && slot_key(chest.slot) === key,
  );
  if (exact.length) return exact;
  return chests.value.filter(
    (chest) =>
      chest.icons.some((drawn) => icons.includes(drawn)) &&
      chest.column?.[0] === address.inventory_type &&
      chest.column[1] === address.page_num &&
      chest.column[2] === address.pos_in_inv[1],
  );
}

// bumped by `wipe_scan`, so a running capture starts over
export const scan_wipes = ref(0);

export const has_progress = computed(
  () =>
    slots.value.size > 0 ||
    Object.keys(edits.value).length > 0 ||
    Object.keys(currency_edits.value).length > 0,
);

// a currency as the page shows it, good with its bands once set by hand; undefined if neither
export function shown_currency(currency: Currency): CurrencyResult | undefined {
  const read = currencies.value.find((x) => x.currency === currency);
  const bands = currency_edits.value[currency];
  if (!bands) return read;
  return {
    ...(read ?? { currency }),
    status: "Good",
    reason: "",
    value: bands.reduce((a, b) => a + b, 0),
    bands: bands as [number, number, number],
  };
}

let latest: ScanResult | null = null;
// edits the scanner has not been told about yet
let unsent: SlotEdit[] = [];

export function take_edits() {
  const out = unsent;
  unsent = [];
  return out;
}

export function reset_scan() {
  latest = null;
  unsent = [];
  slot_images.clear();
  slots.value = new Map();
  chests.value = [];
  currencies.value = [];
  game_pages.value = {};
  seen_pages.value = {};
  edits.value = {};
  currency_edits.value = {};
}

// addresses come out of reactive state, and a proxy cannot be posted to the worker
const plain = (address: SlotAddress): SlotAddress => ({
  ...address,
  pos_in_inv: [...address.pos_in_inv],
});

export function set_edit(edit: SlotEdit) {
  const key = slot_key(edit.address);
  const icon =
    edits.value[key]?.icon ?? slots.value.get(key)?.icon_name_score?.[0];
  edit = { ...edit, address: plain(edit.address), icon: icon ?? null };
  edits.value[key] = edit;
  unsent.push(edit);
}

// forget the slot, edit and all, so the scanner reads it again
export function retry_slot(address: SlotAddress) {
  const key = slot_key(address);
  delete edits.value[key];
  slot_images.delete(key);
  const rest = new Map(slots.value);
  rest.delete(key);
  slots.value = rest;
  unsent.push({
    address: plain(address),
    item: null,
    amount: null,
    tradability: null,
    retry: true,
  });
}

const render_timings: number[] = ((globalThis as any).__scan_renders ??= []);

// called for every scan, so it only merges; the grid is updated a few times a second
export function process_result(result: ScanResult) {
  if (result.full) {
    // edited slots are not in the result, so their images would not come back
    for (const key of [...slot_images.keys()]) {
      if (!edits.value[key]) slot_images.delete(key);
    }
  }
  for (const slot of result.slots) {
    if (slot.image) slot_images.set(slot_key(slot.address), slot.image);
  }
  if (result.chests) chests.value = result.chests;
  for (const [inventory, page] of result.pages) {
    if (game_pages.value[inventory] !== page)
      game_pages.value[inventory] = page;
    if (!seen_pages.value[page_key(inventory, page)])
      seen_pages.value[page_key(inventory, page)] = true;
  }
  latest = result;
  render();
}

const render = useThrottleFn(
  () => {
    if (!latest) return;
    const start = performance.now();
    slots.value = new Map(
      latest.slots.map((slot) => [slot_key(slot.address), slot]),
    );
    currencies.value = latest.currencies;
    render_timings.push(performance.now() - start);
    if (render_timings.length > 5000) render_timings.shift();
  },
  250,
  true,
);
