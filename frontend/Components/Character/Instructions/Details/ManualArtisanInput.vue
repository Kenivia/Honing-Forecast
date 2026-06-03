<script setup lang="ts">
import "@/Components/Character/Instructions/Details/details.css";

const starting_artisan = defineModel<string>("starting_artisan", {
  required: true,
});
const current_chance_percentage = defineModel<string>(
  "current_chance_percentage",
  { required: true },
);

defineProps<{
  base_chance: number;
  optimizer_working: boolean;
  taps_since_last_input: number;
  normal_dist_length: number;
  using_slider: boolean;
  show_hints: boolean;
}>();

const emit = defineEmits<{
  manual_artisan_change: [];
  manual_chance_change: [];
  confirm: [];
}>();
</script>

<template>
  <span class="stat-label">Current artisan energy:</span>
  <div class="flex flex-row">
    <div class="stat-input">
      <input
        class="generic-input number-border w-13"
        :style="{
          backgroundColor: using_slider ? 'transparent' : 'var(--bg-bright)',
        }"
        v-model="starting_artisan"
        inputmode="decimal"
        :disabled="optimizer_working"
        @change="emit('manual_artisan_change')"
      />
    </div>
    <div
      v-if="show_hints"
      class="question-mark"
      v-tooltip.left="
        'You can also input artisan directly here. However, costs will not be auto-deducted'
      "
    />
  </div>
  <div class="button-row">
    <button
      @click="emit('confirm')"
      class="generic-button w-20!"
      :disabled="
        optimizer_working || taps_since_last_input === normal_dist_length - 1
      "
      :style="{
        opacity: taps_since_last_input === normal_dist_length - 1 ? 0.5 : 1,
      }"
    >
      Confirm
    </button>
    <div
      v-if="show_hints"
      class="question-mark"
      v-tooltip.left="
        'Use the slider to update artisan & deduct costs, then press Confirm to re-run the optimizer.'
      "
    />
  </div>
  <span class="stat-label">Current base chance:</span>
  <div class="stat-input">
    <input
      class="generic-input number-border w-10 pr-0!"
      v-model="current_chance_percentage"
      :min="base_chance * 100"
      :max="base_chance * 100 * 2"
      @change="emit('manual_chance_change')"
      :disabled="optimizer_working"
      inputmode="decimal"
      :style="{
        backgroundColor: using_slider ? 'transparent' : 'var(--bg-bright)',
      }"
    />
    <span>%</span>
  </div>
</template>
