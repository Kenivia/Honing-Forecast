<script setup lang="ts">
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { ALT_BANDS, BandPlan, MAIN_BANDS } from "@/Utils/Constants";

// Main / Alt: presets for what leftover mats are worth; "Custom" when the bands match neither.
const { active_profile } = storeToRefs(useRosterStore());
const ROLES: [string, BandPlan][] = [
  ["Main", MAIN_BANDS],
  ["Alt", ALT_BANDS],
];
const same_bands = (a: BandPlan, b: BandPlan) =>
  a.every((value, band) => value === b[band]);
function set_role(plan: BandPlan) {
  active_profile.value.band_values = [...plan];
  active_profile.value.bands_manual = true;
}
</script>

<template>
  <span class="flex flex-row items-center gap-2">
    <button
      v-for="[name, plan] in ROLES"
      :key="name"
      class="generic-button"
      :class="{
        'border-(--gold)! text-(--gold)!': same_bands(
          active_profile.band_values,
          plan,
        ),
      }"
      :aria-pressed="same_bands(active_profile.band_values, plan)"
      @click="set_role(plan)"
    >
      {{ name }}
    </button>
    <span
      v-if="
        ROLES.every(([, plan]) => !same_bands(active_profile.band_values, plan))
      "
      class="text-xs text-(--gold)"
      >Custom</span
    >
  </span>
</template>
