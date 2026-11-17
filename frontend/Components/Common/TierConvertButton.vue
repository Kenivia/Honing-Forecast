<script setup lang="ts">
import { computed } from "vue";
import HoverTip from "@/Components/Common/HoverTip.vue";

const props = defineProps<{
  labelText: string;
  tooltipText?: string | null;
  checkEligibility?: () => boolean | null;
  showTooltipOnlyOnDisabled?: boolean | null;
  warning?: boolean;
}>();

const emit = defineEmits(["change-tier"]);

const eligible = computed(() => {
  return props.checkEligibility ? props.checkEligibility() : true;
});

const activeTooltip = computed(() => {
  if (!props.tooltipText) return null;
  if (props.showTooltipOnlyOnDisabled && eligible.value) return null;
  return props.tooltipText;
});

function handleClick() {
  if (eligible.value) {
    emit("change-tier");
  }
}
</script>

<template>
  <HoverTip :text="activeTooltip" placement="top" class="flex flex-col">
    <button
      :disabled="!eligible"
      class="generic-button tier-button text-wrap!"
      :style="{
        color:
          warning && eligible
            ? 'var(--warning)'
            : !eligible
              ? 'var(--text-very-muted)'
              : 'inherit',
        backgroundColor: 'var(--serca-blue)',
        cursor: !eligible ? 'not-allowed' : 'pointer',
      }"
      @click="handleClick"
    >
      {{ labelText }}
    </button>
  </HoverTip>
</template>

<style scoped>
.tier-button:hover {
  background-color: var(--serca-blue-hover);
}
</style>
