<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { page_key } from "@/Utils/Scans";
import { SlotAddress } from "./LoadStorage";
import {
  edits,
  game_pages,
  roster_cover,
  shown_slot,
  slot_images,
  slot_key,
  slots,
} from "./ScanStore";
import { manifest, roster_covered, roster_saved } from "./Manifest";
import SlotCell from "./SlotCell.vue";
import SlotDashboard from "./SlotDashboard.vue";

defineProps<{ items: string[] }>();

// the three windows left to right as the game lays them out, with the rows of each page
const WINDOWS = [
  { type: "Roster", label: "Roster storage", cols: 6, pages: [10, 10] },
  {
    type: "CharStorage",
    label: "Character storage",
    cols: 10,
    pages: [10, 10, 10, 10],
  },
  { type: "CharInventory", label: "Inventory", cols: 10, pages: [10, 5] },
];
// px a slot may shrink to before the windows wrap
const MIN_SLOT = 28;

const page = reactive<Record<string, number>>({
  Roster: 0,
  CharStorage: 0,
  CharInventory: 0,
});

// the game page each window was last turned to
const followed: Record<string, number> = {};
// The shown page follows the game's: it turns when the page in game does, and a tab clicked by
// hand holds until then.
watch(
  game_pages,
  (pages) => {
    for (const win of WINDOWS) {
      // a new capture starts over
      if (pages[win.type] === undefined) delete followed[win.type];
      else if (pages[win.type] !== followed[win.type]) {
        followed[win.type] = page[win.type] = pages[win.type];
      }
    }
  },
  { deep: true, immediate: true },
);

function addresses(win: (typeof WINDOWS)[number]): SlotAddress[] {
  const page_num = page[win.type];
  const out: SlotAddress[] = [];
  for (let row = 0; row < win.pages[page_num]; row++) {
    for (let col = 0; col < win.cols; col++) {
      out.push({
        inventory_type: win.type,
        page_num,
        pos_in_inv: [row, col],
      });
    }
  }
  return out;
}

const { roster_config, active_profile, all_profiles } =
  storeToRefs(useRosterStore());

// pages taken from the last export, since they were not open in game
const saved = computed(
  () =>
    new Set(
      manifest.value.saved.map((page) => page_key(page.window, page.page)),
    ),
);
const has_saved = (type: string) =>
  [...saved.value].some((key) => key.startsWith(`${type} `));

// who exported roster storage last, and when
const roster_export = computed(() => {
  const pages = roster_config.value.scan_pages.filter(
    (page) =>
      page.window === "Roster" &&
      page.char_id === null &&
      page.roster_id === active_profile.value.roster_id,
  );
  const last = pages.reduce((a, b) => (b.at > a.at ? b : a), pages[0]);
  if (!last) return "";
  const name =
    all_profiles.value.find((profile) => profile.id === last.by)?.char_name ??
    "a deleted character";
  return `${name}, ${new Date(last.at).toLocaleString()}`;
});

const hovered = ref<SlotAddress | null>(null);
const locked = ref<SlotAddress | null>(null);
// the dashboard follows the cursor until a slot is clicked, then stays on that one
const shown = computed(() => locked.value ?? hovered.value);
const is_active = (address: SlotAddress) =>
  !!shown.value && slot_key(shown.value) === slot_key(address);

function toggle_lock(address: SlotAddress) {
  const same = locked.value && slot_key(locked.value) === slot_key(address);
  locked.value = same ? null : address;
}
</script>

<template>
  <div class="flex w-full flex-col gap-3">
    <div class="flex w-full flex-wrap gap-3" @mouseleave="hovered = null">
      <div
        v-for="win in WINDOWS"
        :key="win.type"
        class="flex flex-col gap-1"
        role="group"
        :aria-label="win.label"
        :style="{
          flex: `${win.cols} 1 0`,
          minWidth: `${win.cols * MIN_SLOT}px`,
        }"
      >
        <div class="flex flex-wrap items-center gap-1 text-sm">
          <span class="mr-1 text-(--text-muted)">{{ win.label }}</span>
          <button
            v-for="(_, index) in win.pages"
            :key="index"
            class="generic-button relative"
            :style="{ color: page[win.type] === index ? 'var(--gold)' : '' }"
            @click="page[win.type] = index"
          >
            {{ index + 1 }}
            <svg
              v-if="saved.has(page_key(win.type, index))"
              role="img"
              aria-label="saved"
              viewBox="0 0 16 16"
              class="absolute -top-1 -right-1 h-2.5 w-2.5 fill-(--gold-muted)"
            >
              <path d="M2 1h10l3 3v11H1V1h1zm2 1v4h7V2H4zm1 8v4h6v-4H5z" />
            </svg>
          </button>
          <span
            v-if="has_saved(win.type)"
            class="annotation flex items-center gap-1"
          >
            <svg viewBox="0 0 16 16" class="h-2.5 w-2.5 fill-(--gold-muted)">
              <path d="M2 1h10l3 3v11H1V1h1zm2 1v4h7V2H4zm1 8v4h6v-4H5z" />
            </svg>
            not open yet: last export used
          </span>
          <button
            v-if="win.type === 'Roster' && roster_saved && !roster_covered"
            class="generic-button ml-auto"
            @click="roster_cover = true"
          >
            Use saved
          </button>
        </div>
        <div
          class="relative grid gap-1"
          :style="{
            gridTemplateColumns: `repeat(${win.cols}, minmax(0, 1fr))`,
          }"
        >
          <div
            v-if="win.type === 'Roster' && roster_covered"
            class="absolute inset-0 z-20 flex flex-col items-center justify-center gap-2 bg-(--bg-muted) p-2 text-center text-sm"
            role="group"
            aria-label="Roster storage covered"
          >
            <span>Taken from its last export ({{ roster_export }}).</span>
            <span class="text-(--text-muted)">
              {{
                roster_cover === null
                  ? "The roster's budgets have not changed since. "
                  : ""
              }}Scan it again if anything in it changed.
            </span>
            <button class="generic-button" @click="roster_cover = false">
              Scan roster storage
            </button>
          </div>
          <SlotCell
            v-for="address in addresses(win)"
            :key="slot_key(address)"
            :info="shown_slot(slots.get(slot_key(address)))"
            :edit="edits[slot_key(address)]"
            :image="slot_images.get(slot_key(address))"
            :active="is_active(address)"
            @mouseenter="hovered = address"
            @click="toggle_lock(address)"
          />
        </div>
      </div>
    </div>
    <SlotDashboard :address="shown" :items="items" />
  </div>
</template>
