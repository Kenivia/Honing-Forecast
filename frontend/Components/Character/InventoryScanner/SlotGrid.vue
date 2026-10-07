<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { SlotAddress } from "./LoadStorage";
import {
  edits,
  game_pages,
  shown_slot,
  slot_images,
  slot_key,
  slots,
} from "./ScanStore";
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
        :style="{
          flex: `${win.cols} 1 0`,
          minWidth: `${win.cols * MIN_SLOT}px`,
        }"
      >
        <div class="flex items-center gap-1 text-sm">
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
          class="grid gap-1"
          :style="{
            gridTemplateColumns: `repeat(${win.cols}, minmax(0, 1fr))`,
          }"
        >
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
