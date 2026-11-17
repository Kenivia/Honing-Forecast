<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  ALL_LABELS,
  BAND_COLORS,
  SPECIAL_LEAP_LABELS,
  TIER_LABELS,
  TIER_MATERIALS,
} from "@/Utils/Constants";
import { check_revert_ilevel_ok, get_icon_path } from "@/Utils/Helpers";
import ChestPanels from "@/Components/Common/ChestPanels.vue";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import TierConvertButton from "@/Components/Common/TierConvertButton.vue";
import { chest_gains } from "@/Utils/Chest";
import {
  change_tier,
  tier_label,
  tier_tooltip,
  tier_warning,
} from "../StatusInput/StatusInputUtil";
import { BAND_NAMES, export_manifest, manifest, page_place } from "./Manifest";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import { Currency, SlotAddress } from "./LoadStorage";
import {
  edits,
  shown_currency,
  slot_images,
  slot_key,
  slots,
  WINDOW_NAMES,
} from "./ScanStore";
import { status_border } from "./ScannerUIutils";
import SlotCell from "./SlotCell.vue";

const { active_profile } = storeToRefs(useRosterStore());

const SERCA_ONLY = ALL_LABELS[1].filter(
  (label) => !ALL_LABELS[0].includes(label),
);
const has_serca = (materials: Record<string, number[]>) =>
  SERCA_ONLY.some((label) => materials[label]?.some((amount) => amount > 0));
// Serca materials anywhere, and in this character's own storage or inventory
const serca = computed(() => ({
  any: has_serca(manifest.value.materials),
  own: [...manifest.value.read, ...manifest.value.saved].some(
    (page) =>
      ["CharStorage", "CharInventory"].includes(page.window) &&
      has_serca(page.content.materials),
  ),
}));

// the tab follows the character's tier, or Serca once any is found
const default_tier = () => (serca.value.any ? 1 : active_profile.value.tier);
const tier = ref(default_tier());
watch(
  () =>
    `${active_profile.value.id} ${active_profile.value.tier} ${serca.value.any}`,
  () => (tier.value = default_tier()),
);

// only chests with no choice add, each as it is, of either tier
const gains = computed(() =>
  chest_gains(
    manifest.value.chests.filter((chest) => chest.options.length === 1),
  ),
);

// shown although some slots are not in it
const forced = ref(false);

// missing slots per window as the grid lays them out, currencies apart
const missing = computed(() => {
  const all = manifest.value.missing;
  const currencies = all.filter((slot) => !slot.address);
  // every window keeps its column, empty or not
  const windows = Object.entries(WINDOW_NAMES).map(([type, name]) => ({
    name,
    slots: all.filter((slot) => slot.address?.inventory_type === type),
  }));
  // e.g. "2 slots and 1 currency"
  const n_slots = all.length - currencies.length;
  const n_cur = currencies.length;
  const subject = [
    n_slots && `${n_slots} slot${n_slots === 1 ? "" : "s"}`,
    n_cur && `${n_cur} ${n_cur === 1 ? "currency" : "currencies"}`,
  ]
    .filter(Boolean)
    .join(" and ");
  return { currencies, windows, subject, one: all.length === 1 };
});

const short_place = ({ page_num, pos_in_inv: [row, col] }: SlotAddress) =>
  `Page ${page_num + 1}, row ${row + 1}, col ${col + 1}`;
const currency_border = (name: string) =>
  status_border(shown_currency(name as Currency)?.status ?? "NeedHover", false);

const labels = computed(() => [
  ...TIER_MATERIALS[tier.value].map((mat) => mat.label),
  SPECIAL_LEAP_LABELS[tier.value],
]);

const loose = computed(() => [
  ...manifest.value.random.map((chest) => ({ ...chest, why: "random" })),
  ...manifest.value.unknown.map((chest) => ({
    ...chest,
    why: "contents unknown",
  })),
]);

// pages not open in game, taken as unchanged since the last export
const unopened = computed(() =>
  manifest.value.saved.map((page) => page_place(page.window, page.page)),
);

const exported = ref(false);
function export_to_calc() {
  export_manifest();
  exported.value = true;
}
</script>

<template>
  <div class="w-full border-t border-(--border-main)" />
  <div
    v-if="manifest.missing.length"
    class="card-shell card-body flex w-fit! max-w-full flex-col items-center gap-3 text-sm"
  >
    <span class="text-center text-(--warning)">
      {{ missing.subject }} {{ missing.one ? "is" : "are" }} not read yet or
      still {{ missing.one ? "needs" : "need" }} a hover,
      <template v-if="forced">
        so {{ missing.one ? "it is" : "they are" }} left out of the manifest.
      </template>
      <template v-else>
        so the manifest is hidden. {{ missing.one ? "It" : "They" }} will be
        ignored if you proceed.
      </template>
    </span>

    <div aria-label="Missing slots" class="flex w-full flex-col gap-3">
      <div
        v-if="missing.currencies.length"
        class="flex flex-wrap items-center gap-x-6 gap-y-1"
      >
        <span class="text-(--text-muted)">Currencies</span>
        <div
          v-for="cur in missing.currencies"
          :key="cur.item"
          class="flex items-center gap-2"
        >
          <span
            class="flex h-8 w-8 items-center justify-center"
            :style="currency_border(cur.item)"
          >
            <img
              :src="get_icon_path(cur.item)"
              class="generic-icon h-5 w-5"
              :alt="cur.item"
            />
          </span>
          {{ cur.item }}
          <QuestionMark :text="cur.why" />
        </div>
      </div>
      <div
        class="grid grid-cols-[minmax(0,400px)] gap-6 lg:grid-cols-[repeat(3,minmax(0,400px))]"
      >
        <div
          v-for="win in missing.windows"
          :key="win.name"
          class="grid min-w-0 grid-cols-[auto_auto_minmax(0,1fr)_auto] content-start items-center gap-x-3 gap-y-1"
          role="group"
          :aria-label="win.name"
        >
          <span class="col-span-4 text-(--text-muted)">{{ win.name }}</span>
          <span v-if="!win.slots.length" class="col-span-4 text-(--text-muted)">
            Nothing missing
          </span>
          <template v-for="slot in win.slots" :key="slot_key(slot.address!)">
            <SlotCell
              class="h-8 w-8 cursor-default!"
              :info="slots.get(slot_key(slot.address!))"
              :edit="edits[slot_key(slot.address!)]"
              :image="slot_images.get(slot_key(slot.address!))"
              :active="false"
            />
            <span class="whitespace-nowrap text-(--text-muted)">
              {{ short_place(slot.address!) }}
            </span>
            <span class="truncate" :title="slot.item">{{ slot.item }}</span>
            <span class="flex items-center gap-1 text-(--text-muted)">
              Why <QuestionMark :text="slot.why" />
            </span>
          </template>
        </div>
      </div>
    </div>
    <button v-if="!forced" class="generic-button w-fit" @click="forced = true">
      Show the manifest without these
    </button>
  </div>
  <div class="w-full border-t border-(--border-main)" />
  <div
    v-if="!manifest.missing.length || forced"
    class="flex w-full flex-col items-center gap-3"
  >
    <div
      v-if="active_profile.tier == 0 && serca.own"
      class="card-shell card-body flex w-175! max-w-full flex-col items-center gap-2 text-center text-sm"
    >
      <span class="text-(--warning)">
        This character is set as T4, but Serca materials were found in its
        storage or inventory.
      </span>
      <TierConvertButton
        :labelText="tier_label(active_profile)"
        :tooltipText="tier_tooltip(active_profile)"
        :checkEligibility="
          () => check_revert_ilevel_ok(active_profile) === true
        "
        @change-tier="() => change_tier(active_profile)"
        :show-tooltip-only-on-disabled="false"
        :warning="tier_warning(active_profile)"
      />
    </div>
    <div
      v-if="manifest.assumed || unopened.length"
      class="card-shell card-body flex w-175! max-w-full flex-col items-center gap-1 text-left text-sm"
    >
      <span class="flex w-full flex-row gap-2 text-(--series-gold)">
        Some information from the previous scan was used.
        <QuestionMark
          text="We assume that an item that stayed in the same spot hasn't been swapped out for a different, identical-looking one. If you did swap one, hovering over it should fix it."
        />
      </span>

      <div v-if="unopened.length" class="w-full text-left text-(--text-muted)">
        <div>Not opened in game, taken as unchanged:</div>
        <div v-for="page in unopened" :key="page" class="pl-8">
          - {{ page }}
        </div>
      </div>
    </div>
    <div class="flex max-w-full flex-col items-center gap-2 text-sm">
      <div class="flex flex-wrap items-center justify-center gap-x-4 gap-y-2">
        <button
          class="generic-button px-4! py-2! text-2xl! text-(--achieved)!"
          @click="export_to_calc"
        >
          Export to calculator
        </button>
        <div class="flex max-w-64 flex-col">
          <span v-if="!exported" class="text-(--text-muted)">
            Overwrites the budgets and chests of this character and their roster
            with the below.
          </span>
          <span v-else class="text-(--achieved)"> Exported. </span>
        </div>
      </div>
      <span
        v-if="manifest.missing.length"
        class="w-175 max-w-full text-center text-(--warning)"
      >
        {{ missing.subject }} {{ missing.one ? "is" : "are" }} missing from this
        manifest and {{ missing.one ? "has" : "have" }} been ignored.
      </span>
    </div>

    <div v-if="loose.length" class="card-shell card-body text-sm">
      <div class="card-title">Not included</div>
      <span class="ml-2 text-(--text-muted)">
        Random chests are not supported, open them.
      </span>
      <div v-for="chest in loose" :key="chest.title + chest.why">
        {{ chest.title }} x{{ chest.count.toLocaleString() }}
        <span class="text-(--text-muted)">({{ chest.why }})</span>
      </div>
    </div>

    <div
      class="card-shell outer-grid"
      role="group"
      aria-label="Manifest materials"
      :style="{ '--grid-cols': '200px repeat(3, 230px)' }"
    >
      <div class="mats-row h-fit! border-b border-(--border-main)">
        <div class="flex justify-end gap-1 pr-2 text-sm">
          <button
            v-for="(name, index) in TIER_LABELS"
            :key="index"
            class="generic-button"
            :class="{ 'bg-(--gold-muted)!': tier === index }"
            @click="tier = index"
          >
            {{ name }}
          </button>
        </div>
        <span
          v-for="(name, band) in BAND_NAMES"
          :key="band"
          class="text-left"
          :style="{ color: `var(${BAND_COLORS[band]})` }"
        >
          {{ name }}
        </span>
      </div>
      <div
        v-for="label in labels"
        :key="label"
        class="mats-row"
        role="group"
        :aria-label="label"
      >
        <span
          class="flex items-center justify-end gap-1.5 pr-2 text-right text-[13px] text-(--text-muted)"
        >
          {{ label }}
          <img
            :src="get_icon_path(label)"
            class="generic-icon h-8.5 w-8.5"
            :alt="label"
          />
        </span>
        <MaterialCell
          v-for="(name, band) in BAND_NAMES"
          :key="band"
          :input_column="{ [label]: manifest.materials[label][band] }"
          :label="label"
          :aria_name="`${label} ${name}`"
          :input_color="BAND_COLORS[band]"
          :input_width="100"
          :justify_left="true"
          :plus="[
            { amount: manifest.incoming[label][band], text: 'from T4' },
            { amount: gains[label]?.[band] ?? 0, text: 'from chests' },
          ]"
        />
      </div>
    </div>

    <ChestPanels
      :selection="manifest.chests.filter((chest) => !chest.obtain_all)"
      :obtain_all="manifest.chests.filter((chest) => chest.obtain_all)"
    />
  </div>
</template>
