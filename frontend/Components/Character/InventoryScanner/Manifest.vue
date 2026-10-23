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
import { BAND_NAMES, export_manifest, manifest } from "./Manifest";

const { active_profile } = storeToRefs(useRosterStore());

const SERCA_ONLY = ALL_LABELS[1].filter(
  (label) => !ALL_LABELS[0].includes(label),
);
const has_serca = (materials: Record<string, number[]>) =>
  SERCA_ONLY.some((label) => materials[label]?.some((amount) => amount > 0));
// Serca materials anywhere, and in this character's own storage or inventory
const serca = computed(() => ({
  any: has_serca(manifest.value.materials),
  own: [
    ...manifest.value.read.map((page) => ({ ...page, ...page.content })),
    ...manifest.value.saved,
  ].some(
    (page) =>
      ["CharStorage", "CharInventory"].includes(page.window) &&
      has_serca(page.materials),
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

const exported = ref(false);
function export_to_calc() {
  export_manifest();
  exported.value = true;
}
</script>

<template>
  <div class="w-full border-t border-(--border-main)" />
  <div
    v-if="manifest.missing.length && !forced"
    class="card-shell card-body flex w-175! max-w-full flex-col items-center gap-2 text-center text-sm"
  >
    <span class="text-(--warning)">
      {{ manifest.missing.length }} slot(s) or currencies are not read yet or
      still need a hover, so the manifest is hidden. These slots will be ignored
      if you proceed.
    </span>

    <ul aria-label="Missing slots" class="flex flex-col gap-1">
      <li v-for="slot in manifest.missing" :key="slot.place + slot.item">
        {{ slot.place }}: {{ slot.item }}
        <span class="text-(--text-muted)">{{ slot.why }}</span>
      </li>
    </ul>
    <button class="generic-button w-fit" @click="forced = true">
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
            Overwrites the budgets and chests of this character and its roster.
          </span>
          <span v-else class="text-(--achieved)"> Exported. </span>
        </div>
      </div>
      <span
        v-if="manifest.missing.length"
        class="w-175 max-w-full text-center text-(--warning)"
      >
        {{ manifest.missing.length }} slot(s) or currencies are missing from
        this manifest. These slots have been ignored.
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
