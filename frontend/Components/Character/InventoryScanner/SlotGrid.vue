<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { page_key } from "@/Utils/Scans";
import { Currency, SlotAddress } from "./LoadStorage";
import { edits, game_pages, slot_images, slot_key, slots } from "./ScanStore";
import { manifest } from "./Manifest";
import CurrencyBar from "./CurrencyBar.vue";
import CurrencyDashboard from "./CurrencyDashboard.vue";
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

const { all_profiles } = storeToRefs(useRosterStore());

// pages not open in game during this capture, taken from the last export, by `page_key`
const saved = computed(() =>
  Object.fromEntries(
    manifest.value.saved.map((page) => [
      page_key(page.window, page.page),
      page,
    ]),
  ),
);

// the page shown in a window, by `page_key`
const shown_page = (type: string) => page_key(type, page[type]);

// who exported the page shown in a window last, and when
function last_export(type: string) {
  const last = saved.value[shown_page(type)];
  const name =
    all_profiles.value.find((profile) => profile.id === last.by)?.char_name ??
    "a deleted character";
  return `${name}, ${new Date(last.at).toLocaleString()}`;
}

// a slot, or a currency by name
type Shown = SlotAddress | Currency;
const shown_key = (x: Shown) => (typeof x === "string" ? x : slot_key(x));

const hovered = ref<Shown | null>(null);
const locked = ref<Shown | null>(null);
// the dashboard follows the cursor until a slot or currency is clicked, then stays on that one
const shown = computed(() => locked.value ?? hovered.value);
const is_active = (x: Shown) =>
  !!shown.value && shown_key(shown.value) === shown_key(x);

function toggle_lock(x: Shown) {
  locked.value = is_active(x) && locked.value ? null : x;
}
</script>

<template>
  <div class="flex w-full flex-col items-center gap-3">
    <div
      class="card-shell card-body flex w-full! flex-col gap-3"
      @mouseleave="hovered = null"
    >
      <CurrencyBar
        :active="typeof shown === 'string' ? shown : null"
        @hover="hovered = $event"
        @pick="toggle_lock"
      />
      <div class="flex w-full flex-wrap gap-3">
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
              class="generic-button"
              :style="{ color: page[win.type] === index ? 'var(--gold)' : '' }"
              @click="page[win.type] = index"
            >
              {{ index + 1 }}
            </button>
          </div>
          <div
            class="relative grid gap-1"
            :style="{
              gridTemplateColumns: `repeat(${win.cols}, minmax(0, 1fr))`,
            }"
          >
            <div
              v-if="saved[shown_page(win.type)]"
              class="absolute inset-0 z-20 flex flex-col items-center justify-center gap-2 bg-(--bg-muted) p-2 text-center text-sm"
              role="group"
              :aria-label="`${win.label} covered`"
            >
              <span>
                Not opened in game yet, so the previous scan of this page is
                used.
              </span>
              <span class="text-(--text-muted)"
                >Exported on {{ last_export(win.type) }}.</span
              >
            </div>
            <SlotCell
              v-for="address in addresses(win)"
              :key="slot_key(address)"
              :info="slots.get(slot_key(address))"
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
    <CurrencyDashboard
      v-if="typeof shown === 'string'"
      class="w-[1032px]! max-w-full"
      :currency="shown"
    />
    <SlotDashboard
      v-else
      class="w-[1032px]! max-w-full"
      :address="shown"
      :items="items"
    />
  </div>
</template>
