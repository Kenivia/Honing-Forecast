<script setup lang="ts">
import { computed } from "vue";
import Button from "primevue/button";

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
  <Button
    v-tooltip.top="activeTooltip"
    :unstyled="true"
    :disabled="!eligible"
    :label="labelText"
    :class="[
      !eligible ? 'cursor-not-allowed' : 'cursor-pointer',
      warning && eligible ? 'text-(--warning)' : '',
      'bg-(--serca-blue)',
      'barebone-button',
    ]"
    @click="handleClick"
  />
</template>
