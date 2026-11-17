<script setup lang="ts">
import { ref } from "vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  BAND_COLORS,
  BAND_LABELS,
  BAND_VALUES,
  BAND_VALUE_LABELS,
  BandValue,
  band_value_allowed,
} from "@/Utils/Constants";
import RoleToggle from "@/Components/Character/RoleToggle.vue";

// What a material the character owns but does not end up spending is worth: the Main / Alt
// presets, and with Customise a dropdown per band. Calc.vue watches band_values and restarts
// the workers. The Customise button sits at the end of the first line so it never moves.
const { active_profile } = storeToRefs(useRosterStore());
const editing = ref(false);

function set_band(band: number, event: Event) {
  active_profile.value.band_values[band] = (event.target as HTMLSelectElement)
    .value as BandValue;
}
</script>

<template>
  <div class="flex w-full flex-col gap-1 pl-2 text-sm">
    <div class="flex flex-row items-center justify-between gap-2">
      <span class="flex flex-row items-center gap-2">
        <span class="text-(--text-muted)">Treating as</span>
        <RoleToggle />
      </span>
      <button
        class="w-20 shrink-0 text-right text-(--text-muted) underline hover:text-(--text-bright)"
        @click="editing = !editing"
      >
        {{ editing ? "Done" : "Customise" }}
      </button>
    </div>
    <template v-if="editing">
      <div
        class="grid w-fit grid-cols-[auto_auto] items-center gap-x-2 gap-y-0.5"
      >
        <template v-for="(label, band) in BAND_LABELS" :key="label">
          <span :style="{ color: `var(${BAND_COLORS[band]})` }">{{
            label
          }}</span>
          <select
            class="selector"
            :aria-label="`${label} leftover value`"
            :value="active_profile.band_values[band]"
            @change="set_band(band, $event)"
          >
            <option
              v-for="value in BAND_VALUES"
              :key="value"
              :value="value"
              :disabled="
                !band_value_allowed(active_profile.band_values, band, value)
              "
            >
              {{ BAND_VALUE_LABELS[value] }}
            </option>
          </select>
        </template>
      </div>
      <span class="annotation text-left text-xs! leading-4!">
        We consider every possible outcome. Any leftover material is converted
        to gold according to these settings.
      </span>
    </template>
  </div>
</template>
