<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  BAND_COLORS,
  SPECIAL_LEAP_LABELS,
  TIER_LABELS,
  TIER_MATERIALS,
} from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import ChestPanels from "@/Components/Common/ChestPanels.vue";
import {
  BAND_NAMES,
  convert,
  convert_owner,
  export_manifest,
  manifest,
  page_place,
} from "./Manifest";

const { active_profile, all_profiles } = storeToRefs(useRosterStore());

// band of each toggle, in the order they are shown
const TOGGLES = [1, 2, 0];

const tier = ref(active_profile.value.tier);
// a T4 character has no use for its own T4 materials as Serca ones
watch(
  () => `${active_profile.value.id} ${active_profile.value.tier}`,
  (owner) => {
    tier.value = active_profile.value.tier;
    // the toggles survive leaving the page, but not another character or tier
    if (convert_owner.value === owner) return;
    convert_owner.value = owner;
    convert.value = [active_profile.value.tier === 1, false, false];
  },
  { immediate: true },
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

const name_of = (id: string) =>
  all_profiles.value.find((profile) => profile.id === id)?.char_name ??
  "a deleted character";
const when = (at: number) => new Date(at).toLocaleString();

// the saved pages this manifest takes in place of ones not seen, a currency's two parts as one
const saved = computed(() => {
  const out = new Map<string, string>();
  for (const page of manifest.value.saved) {
    out.set(
      page_place(page.window, page.page),
      `scanned on ${name_of(page.by)}, ${when(page.at)}`,
    );
  }
  return [...out];
});
// the other characters of the roster, with when their pages were last exported
const others = computed(() => {
  const out = new Map<string, number>();
  for (const page of manifest.value.others) {
    out.set(page.char_id, Math.max(out.get(page.char_id) ?? 0, page.at));
  }
  return [...out].map(([id, at]) => `${name_of(id)}, ${when(at)}`);
});

const exported = ref(false);
function export_to_calc() {
  export_manifest();
  exported.value = true;
}
</script>

<template>
  <div
    v-if="manifest.missing.length && !forced"
    class="card-shell card-body flex flex-col gap-2 text-sm"
  >
    <span class="text-(--warning)">
      {{ manifest.missing.length }} slot(s) or currencies are not read yet or
      still need a hover, so the manifest is hidden. Exporting sets those slots
      to irrelevant.
    </span>
    <button class="generic-button w-fit" @click="forced = true">
      Show the manifest without them
    </button>
    <ul aria-label="Missing slots" class="flex flex-col gap-1">
      <li v-for="slot in manifest.missing" :key="slot.place + slot.item">
        {{ slot.place }}: {{ slot.item }}
        <span class="text-(--text-muted)">{{ slot.why }}</span>
      </li>
    </ul>
  </div>
  <div v-else class="flex w-full flex-col gap-3">
    <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
      <label
        v-for="band in TOGGLES"
        :key="band"
        class="flex items-center gap-1"
        :class="{ 'opacity-50': band === 0 && active_profile.tier === 0 }"
      >
        <input
          v-model="convert[band]"
          type="checkbox"
          :disabled="band === 0 && active_profile.tier === 0"
        />
        Convert {{ BAND_NAMES[band].toLowerCase() }} T4 to Serca
      </label>
      <button class="generic-button" @click="export_to_calc">
        Export to calculator
      </button>
      <span v-if="exported" class="text-(--text-muted)">
        Exported. The budgets and chests of this character and its roster were
        overwritten, and snapshots taken before and after.
      </span>
    </div>
    <span v-if="manifest.missing.length" class="text-sm text-(--warning)">
      {{ manifest.missing.length }} slot(s) or currencies are missing from this
      manifest. Exporting sets those slots to irrelevant.
    </span>

    <div class="flex gap-1 text-sm">
      <button
        v-for="(name, index) in TIER_LABELS"
        :key="index"
        class="generic-button"
        :style="{ color: tier === index ? 'var(--gold)' : '' }"
        @click="tier = index"
      >
        {{ name }}
      </button>
    </div>

    <div class="flex flex-wrap items-start gap-3">
      <div
        class="card-shell outer-grid"
        role="group"
        aria-label="Manifest materials"
        :style="{ '--grid-cols': '200px repeat(3, 230px)' }"
      >
        <div class="mats-row h-fit! border-b border-(--border-main)">
          <span></span>
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
          <div
            v-for="(name, band) in BAND_NAMES"
            :key="band"
            class="flex h-full items-center gap-1"
          >
            <span
              class="w-25 pl-1"
              :aria-label="`${label} ${name}`"
              :style="{ color: `var(${BAND_COLORS[band]})` }"
            >
              {{ manifest.materials[label][band].toLocaleString() }}
            </span>
            <span
              v-if="manifest.incoming[label][band]"
              class="annotation whitespace-nowrap"
            >
              +{{ manifest.incoming[label][band].toLocaleString() }} from T4
            </span>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-3">
        <ChestPanels
          :selection="manifest.chests.filter((chest) => !chest.obtain_all)"
          :obtain_all="manifest.chests.filter((chest) => chest.obtain_all)"
        />

        <div v-if="loose.length" class="card-shell card-body text-sm">
          <div class="card-title">Not included</div>
          <span class="text-(--text-muted)">
            Open these in game, then scan again.
          </span>
          <div v-for="chest in loose" :key="chest.title + chest.why">
            {{ chest.title }} x{{ chest.count.toLocaleString() }}
            <span class="text-(--text-muted)">({{ chest.why }})</span>
          </div>
        </div>

        <div
          v-if="saved.length || others.length"
          class="card-shell card-body text-sm"
          role="group"
          aria-label="Saved pages"
        >
          <div class="card-title">From earlier exports</div>
          <span class="text-(--text-muted)">
            Not open in game this time, so taken as last exported. Open them if
            anything moved.
          </span>
          <div v-for="[place, from] in saved" :key="place">
            {{ place }}
            <span class="text-(--text-muted)">{{ from }}</span>
          </div>
          <template v-if="others.length">
            <div class="pt-1 text-(--text-muted)">
              Roster bound and tradable from other characters:
            </div>
            <div v-for="other in others" :key="other">{{ other }}</div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
