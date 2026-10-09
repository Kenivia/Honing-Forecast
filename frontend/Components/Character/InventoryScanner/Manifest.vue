<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  BAND_COLORS,
  MATERIALS,
  SPECIAL_LEAP_LABELS,
  TIER_LABELS,
  TIER_MATERIALS,
} from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import { added_chests, chest_overrides } from "./ScanStore";
import {
  cell_text,
  convert,
  convert_owner,
  manifest,
  set_cell_text,
} from "./Manifest";

const { active_profile } = storeToRefs(useRosterStore());

const BANDS = ["Char bound", "Roster bound", "Tradable"];
// band of each toggle, in the order they are shown
const TOGGLES = [1, 2, 0];

const tier = ref(active_profile.value.tier);
// a T4 character has no use for its own T4 materials as Serca ones
watch(
  () => `${active_profile.value.char_name} ${active_profile.value.tier}`,
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
const tier_chests = computed(() =>
  manifest.value.chests.filter((chest) =>
    chest.options.some((option) =>
      Object.keys(option).some((label) =>
        MATERIALS[label].tiers.includes(tier.value),
      ),
    ),
  ),
);
const loose = computed(() => [
  ...manifest.value.random.map((chest) => ({ ...chest, why: "random" })),
  ...manifest.value.unknown.map((chest) => ({
    ...chest,
    why: "contents unknown",
  })),
]);

const amount_text = (amount: number) =>
  amount.toLocaleString("en-US", { maximumFractionDigits: 1 });

// a select-one chest typed in by hand; an option is one material
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
  added_chests.value.push({
    key: `added|${Date.now()}`,
    title: draft.value.title.trim() || "Added chest",
    band: draft.value.band,
    count: Math.max(1, Math.floor(Number(draft.value.count)) || 1),
    options: draft_options.value.map((option) => ({
      [option.label]: Number(option.amount),
    })),
  });
  draft.value = fresh_chest();
}

function remove_chest(key: string) {
  added_chests.value = added_chests.value.filter((chest) => chest.key !== key);
  delete chest_overrides.value[key];
}

function set_chest_count(key: string, event: Event) {
  const value = parseInt((event.target as HTMLInputElement).value);
  if (isFinite(value)) chest_overrides.value[key] = Math.max(0, value);
  else delete chest_overrides.value[key];
}
</script>

<template>
  <div
    v-if="manifest.missing.length && !forced"
    class="card-shell card-body flex flex-col gap-2 text-sm"
  >
    <span class="text-(--warning)">
      {{ manifest.missing.length }} slot(s) are not read yet or still need a
      hover, so the manifest is hidden.
    </span>
    <button class="generic-button w-fit" @click="forced = true">
      Show the manifest without them
    </button>
    <ul aria-label="Missing slots" class="flex flex-col gap-1">
      <li v-for="slot in manifest.missing" :key="slot.place">
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
        Convert {{ BANDS[band].toLowerCase() }} T4 to Serca
      </label>
    </div>
    <span v-if="manifest.missing.length" class="text-sm text-(--warning)">
      {{ manifest.missing.length }} slot(s) are missing from this manifest.
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
        :style="{ '--grid-cols': '200px repeat(3, 190px)' }"
      >
        <div class="mats-row h-fit! border-b border-(--border-main)">
          <span></span>
          <span
            v-for="(name, band) in BANDS"
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
            v-for="(name, band) in BANDS"
            :key="band"
            class="flex h-full items-center gap-1"
          >
            <input
              type="text"
              class="generic-input w-25 pl-1"
              placeholder="?"
              :aria-label="`${label} ${name}`"
              :style="{ color: `var(${BAND_COLORS[band]})` }"
              :value="cell_text(label, band)"
              @change="
                set_cell_text(
                  label,
                  band,
                  ($event.target as HTMLInputElement).value,
                );
                ($event.target as HTMLInputElement).value = cell_text(
                  label,
                  band,
                );
              "
            />
            <span v-if="manifest.incoming[label][band]" class="annotation">
              +{{ manifest.incoming[label][band].toLocaleString() }} from T4
            </span>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-3">
        <div class="card-shell card-body text-sm">
          <div class="card-title">Selection chests</div>
          <span v-if="!tier_chests.length" class="text-(--text-muted)">
            None read for this tier.
          </span>
          <div
            v-for="chest in tier_chests"
            :key="chest.key"
            class="flex flex-wrap items-center gap-2 border-b border-(--border-very-muted) py-1"
          >
            <input
              type="number"
              min="0"
              class="generic-input h-7! w-16 pl-1"
              :aria-label="`${chest.title} count`"
              :value="chest.count"
              @change="set_chest_count(chest.key, $event)"
            />
            <span class="flex flex-col">
              {{ chest.title }}
              <span
                class="annotation"
                :style="{ color: `var(${BAND_COLORS[chest.band]})` }"
              >
                {{ BANDS[chest.band] }}
              </span>
            </span>
            <template v-for="(option, index) in chest.options" :key="index">
              <span class="text-(--text-very-muted)">
                {{ index ? "or" : ":" }}
              </span>
              <span
                v-for="(amount, label) in option"
                :key="label"
                class="flex items-center gap-1"
              >
                <img
                  :src="get_icon_path(label)"
                  class="generic-icon h-6 w-6"
                  :alt="label"
                  :title="label"
                />
                x{{ amount_text(amount) }}
              </span>
            </template>
            <button
              v-if="chest.key.startsWith('added|')"
              class="generic-button"
              @click="remove_chest(chest.key)"
            >
              Remove
            </button>
          </div>
        </div>

        <div class="card-shell card-body flex flex-col gap-2 text-sm">
          <div class="card-title">Add a selection chest</div>
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
              <option v-for="(name, band) in BANDS" :key="band" :value="band">
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
              <option v-for="label in labels" :key="label">{{ label }}</option>
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
      </div>
    </div>
  </div>
</template>
