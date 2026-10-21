<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { page_key } from "@/Utils/Scans";
import { SlotAddress } from "./LoadStorage";
import {
  edits,
  game_pages,
  shown_slot,
  slot_images,
  slot_key,
  slots,
  window_cover,
} from "./ScanStore";
import { COVERABLE, cover_pages, covered, manifest } from "./Manifest";
import CurrencyBar from "./CurrencyBar.vue";
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

// who exported a coverable window last, and when
function last_export(type: string) {
  const owner = type === "Roster" ? null : active_profile.value.id;
  const pages = roster_config.value.scan_pages.filter(
    (page) =>
      page.window === type &&
      page.char_id === owner &&
      page.roster_id === active_profile.value.roster_id,
  );
  const last = pages.reduce((a, b) => (b.at > a.at ? b : a), pages[0]);
  if (!last) return "";
  const name =
    all_profiles.value.find((profile) => profile.id === last.by)?.char_name ??
    "a deleted character";
  return `${name}, ${new Date(last.at).toLocaleString()}`;
}

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
  <div class="flex w-full flex-col items-center gap-3">
    <div class="card-shell card-body flex w-full! flex-col gap-3">
      <CurrencyBar />
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
            <!-- covers every page of a coverable window with its last export -->
            <button
              v-if="COVERABLE.includes(win.type) && cover_pages[win.type].size"
              class="generic-button ml-2 flex items-center gap-1"
              :class="{ 'bg-(--gold-muted)!': covered[win.type] }"
              :aria-pressed="covered[win.type]"
              @click="window_cover[win.type] = !covered[win.type]"
            >
              <svg
                viewBox="0 0 16 16"
                class="h-2.5 w-2.5"
                :class="
                  covered[win.type]
                    ? 'fill-(--text-bright)'
                    : 'fill-(--gold-muted)'
                "
              >
                <path d="M2 1h10l3 3v11H1V1h1zm2 1v4h7V2H4zm1 8v4h6v-4H5z" />
              </svg>
              {{
                covered[win.type]
                  ? "Using most recent scan"
                  : "Use most recent scan"
              }}
            </button>
            <span
              v-else-if="has_saved(win.type)"
              class="annotation ml-2 flex items-center gap-1"
            >
              <svg viewBox="0 0 16 16" class="h-2.5 w-2.5 fill-(--gold-muted)">
                <path d="M2 1h10l3 3v11H1V1h1zm2 1v4h7V2H4zm1 8v4h6v-4H5z" />
              </svg>
              Using most recent scan
            </span>
          </div>
          <div
            class="relative grid gap-1"
            :style="{
              gridTemplateColumns: `repeat(${win.cols}, minmax(0, 1fr))`,
            }"
          >
            <div
              v-if="covered[win.type]"
              class="absolute inset-0 z-20 flex flex-col items-center justify-center gap-2 bg-(--bg-muted) p-2 text-center text-sm"
              role="group"
              :aria-label="`${win.label} covered`"
            >
              <span>Using most recent scan ({{ last_export(win.type) }}).</span>
              <span class="text-(--text-muted)">
                {{
                  window_cover[win.type] === null
                    ? "You don't need to re-scan if nothing here has changed. "
                    : ""
                }}
              </span>
              <button
                class="generic-button"
                @click="window_cover[win.type] = false"
              >
                Scan {{ win.label.toLowerCase() }}
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
    </div>
    <SlotDashboard
      class="w-[1055px]! max-w-full"
      :address="shown"
      :items="items"
    />
  </div>
</template>
