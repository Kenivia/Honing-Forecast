<script setup lang="ts">
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  BAND_NAMES,
  SPECIAL_LEAP_LABELS,
  TIER_MATERIALS,
} from "@/Utils/Constants";
import ChestPanels from "@/Components/Common/ChestPanels.vue";

// The chests as stored, both tiers in them: this character's own and its roster's. A chest
// bound to a character goes to the character, any other to the roster.
const { active_profile, active_chests, roster_config } =
  storeToRefs(useRosterStore());

const roster_list = () =>
  (roster_config.value.roster_chests[active_profile.value.roster_id] ??= []);
// the list a chest is in and where, if it is in one
function find(key: string) {
  for (const list of [
    active_profile.value.chests,
    roster_config.value.roster_chests[active_profile.value.roster_id] ?? [],
  ]) {
    const index = list.findIndex((chest) => chest.key === key);
    if (index >= 0) return { list, index };
  }
}

function set_count(key: string, event: Event) {
  const input = event.target as HTMLInputElement;
  const value = parseInt(input.value);
  const at = find(key);
  if (!at) return;
  if (isFinite(value)) at.list[at.index].count = Math.max(0, value);
  // the box shows what was kept, even when that did not change
  input.value = String(at.list[at.index].count);
}

function remove(key: string) {
  const at = find(key);
  at?.list.splice(at.index, 1);
}

const labels = computed(() => [
  ...TIER_MATERIALS[active_profile.value.tier].map((mat) => mat.label),
  SPECIAL_LEAP_LABELS[active_profile.value.tier],
]);

// a chest typed in by hand; an option is one material
const fresh_chest = () => ({
  title: "",
  count: 1,
  band: 1,
  options: [{ label: "", amount: "" }],
});
const draft = ref(fresh_chest());
const draft_options = computed(() =>
  draft.value.options.filter(
    (option) => option.label && Number(option.amount) > 0,
  ),
);

function add_chest() {
  const { band } = draft.value;
  (band ? roster_list() : active_profile.value.chests).push({
    key: `added|${crypto.randomUUID()}`,
    title: draft.value.title.trim() || "Added chest",
    band,
    count: Math.max(1, Math.floor(Number(draft.value.count)) || 1),
    options: draft_options.value.map((option) => ({
      [option.label]: Number(option.amount),
    })),
    obtain_all: false,
  });
  draft.value = fresh_chest();
}
</script>

<template>
  <ChestPanels
    :selection="active_chests.filter((chest) => !chest.obtain_all)"
    :obtain_all="active_chests.filter((chest) => chest.obtain_all)"
    :editing="true"
    @count="set_count"
    @remove="remove"
  >
    <div class="flex flex-col gap-2 pt-2">
      <div class="card-title">Add a chest</div>
      <div class="flex flex-wrap items-center gap-2">
        <input
          v-model="draft.count"
          type="number"
          min="1"
          class="generic-input h-7! w-16 pl-1"
          aria-label="New chest count"
        />
        <input
          v-model="draft.title"
          type="text"
          class="generic-input h-7! w-48 pl-1"
          placeholder="Name (optional)"
          aria-label="New chest name"
        />
        <select
          v-model="draft.band"
          class="selector"
          aria-label="New chest ownership"
        >
          <option v-for="(name, band) in BAND_NAMES" :key="band" :value="band">
            {{ name }}
          </option>
        </select>
      </div>
      <div
        v-for="(option, index) in draft.options"
        :key="index"
        class="flex items-center gap-2"
      >
        <span class="w-4 text-(--text-very-muted)">
          {{ index ? "or" : "" }}
        </span>
        <select
          v-model="option.label"
          class="selector"
          :aria-label="`Option ${index + 1} material`"
        >
          <option value="" disabled>Material</option>
          <option v-for="label in labels" :key="label">
            {{ label }}
          </option>
        </select>
        <input
          v-model="option.amount"
          type="number"
          min="1"
          class="generic-input h-7! w-24 pl-1"
          placeholder="Amount"
          :aria-label="`Option ${index + 1} amount`"
        />
        <button
          v-if="draft.options.length > 1"
          class="generic-button"
          @click="draft.options.splice(index, 1)"
        >
          Remove
        </button>
      </div>
      <div class="flex gap-2">
        <button
          class="generic-button"
          @click="draft.options.push({ label: '', amount: '' })"
        >
          Add option
        </button>
        <button
          class="generic-button"
          :class="{
            'cursor-not-allowed! opacity-50': !draft_options.length,
          }"
          :disabled="!draft_options.length"
          @click="add_chest"
        >
          Add chest
        </button>
      </div>
    </div>
  </ChestPanels>
</template>
