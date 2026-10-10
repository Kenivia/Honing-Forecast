import { computed, ref, shallowRef } from "vue";
import { useThrottleFn } from "@vueuse/core";
import {
  Chest,
  OneIconConfig,
  ScanResult,
  SlotAddress,
  SlotResult,
} from "./LoadStorage";
import type { ManifestChest } from "./Manifest";

// What the slot grid shows. Module state: it outlives the page and the scanner worker, so the
// grid stays up and editable after capture stops. Only a new capture wipes it.

export interface SlotEdit {
  address: SlotAddress;
  item: string | null;
  amount: number | null;
  tradability: string | null;
  retry?: boolean;
}

const WINDOW_NAMES = {
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
// the page each located window shows in game
export const game_pages = ref<Record<string, number>>({});
// what the user typed in; the scanner leaves these slots alone
export const edits = ref<Record<string, SlotEdit>>({});
// images only arrive when they change, and the same objects are reused so canvases are not redrawn
export const slot_images = new Map<string, OneIconConfig>();

// Taken on trust for a whole window where no tooltip said otherwise. The scanner is not told, so
// a tooltip it does see still wins.
export const assume_roster_tradable = ref(false);
// character storage and the inventory
export const assume_char_bound = ref(false);

export const assumed_tradability = (address: SlotAddress) =>
  address.inventory_type === "Roster" && assume_roster_tradable.value
    ? "Tradable"
    : address.inventory_type !== "Roster" && assume_char_bound.value
      ? "CharBound"
      : null;

// a slot as the page shows it: an unknown tradability filled in by what is assumed for its window
export function shown_slot(slot: SlotResult | undefined) {
  if (slot?.status !== "NeedTradability") return slot;
  const assumed = assumed_tradability(slot.address);
  if (!assumed) return slot;
  return { ...slot, status: "Good" as const, reason: "", tradability: assumed };
}

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

// what the user typed into the manifest, over what was scanned
export const material_overrides = ref<Record<string, string>>({});
export const chest_overrides = ref<Record<string, number>>({});
// select-one chests the user added to the manifest by hand
export const added_chests = ref<ManifestChest[]>([]);
// keys of the chests the user took out of the manifest
export const deleted_chests = ref<string[]>([]);

export const has_progress = computed(
  () => slots.value.size > 0 || Object.keys(edits.value).length > 0,
);

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
  game_pages.value = {};
  edits.value = {};
  material_overrides.value = {};
  chest_overrides.value = {};
  added_chests.value = [];
  deleted_chests.value = [];
}

// addresses come out of reactive state, and a proxy cannot be posted to the worker
const plain = (address: SlotAddress): SlotAddress => ({
  ...address,
  pos_in_inv: [...address.pos_in_inv],
});

export function set_edit(edit: SlotEdit) {
  edit = { ...edit, address: plain(edit.address) };
  edits.value[slot_key(edit.address)] = edit;
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
    render_timings.push(performance.now() - start);
    if (render_timings.length > 5000) render_timings.shift();
  },
  250,
  true,
);
