<script setup lang="ts">
// The three ownership bands as one ladder, left to right, ending in the market price.
// Clicking a band cycles what a leftover unit of it is worth; the plan can never
// decrease, so raising a band raises the ones above it and lowering lowers those below.
import {
  BAND_COLORS,
  BAND_LABELS,
  BAND_VALUE_LABELS,
  BandPlan,
  cycle_band_value,
} from "@/Utils/Constants";
import QuestionMark from "@/Components/Common/QuestionMark.vue";

const props = defineProps<{ plan: BandPlan }>();
const emit = defineEmits<{ (e: "update", plan: BandPlan): void }>();

const tooltip = `What a material you own but don't end up spending is worth.
Click a band to cycle it. The optimizer spends to minimise the total,
so crediting more makes it willing to buy more.`;
</script>

<template>
  <div class="flex flex-row items-center gap-2">
    <span class="text-sm text-nowrap text-(--text-muted)"
      >Leftover mats are worth</span
    >
    <QuestionMark :text="tooltip" />
    <div class="flex min-w-0 flex-1 flex-row">
      <button
        v-for="(label, band) in BAND_LABELS"
        :key="label"
        type="button"
        class="band-segment"
        :style="{ color: `var(${BAND_COLORS[band]})` }"
        :aria-label="`${label} leftover value`"
        @click="emit('update', cycle_band_value(props.plan, band))"
      >
        <span class="text-xs text-(--text-very-muted)">{{ label }}</span>
        <span class="text-sm text-nowrap">{{
          BAND_VALUE_LABELS[props.plan[band]]
        }}</span>
      </button>
      <div class="band-segment band-market">
        <span class="text-xs text-(--text-very-muted)">Beyond</span>
        <span class="text-sm text-nowrap text-(--gold)">buy @ market</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.band-segment {
  display: flex;
  flex: 1 1 0;
  min-width: 0;
  flex-direction: column;
  align-items: center;
  padding: 2px 4px;
  border: 1px solid var(--border-muted);
  border-right: none;
  background-color: var(--bg-main);
  cursor: pointer;
  user-select: none;
}
.band-segment:first-child {
  border-top-left-radius: 4px;
  border-bottom-left-radius: 4px;
}
.band-segment:hover {
  background-color: var(--bg-very-bright);
}
.band-market {
  border-right: 1px solid var(--border-muted);
  border-top-right-radius: 4px;
  border-bottom-right-radius: 4px;
  background-color: var(--bg-muted);
  cursor: default;
}
.band-market:hover {
  background-color: var(--bg-muted);
}
</style>
