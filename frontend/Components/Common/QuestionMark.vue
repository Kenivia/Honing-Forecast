<script setup lang="ts">
import { computed } from "vue";
import { useAttrs } from "vue";

const attrs = useAttrs();

export interface Props {
  text: string;
  position?: "left" | "right" | "top" | "bottom" | "none";
}

withDefaults(defineProps<Props>(), {
  position: "none",
});

const additionalClasses = computed(() => {
  const classes = attrs.class;
  return typeof classes === "string" ? classes : "";
});
</script>

<template>
  <div
    v-if="position === 'none'"
    :class="`question-mark ${additionalClasses}`"
    v-tooltip="text"
    v-once
  />
  <div
    v-else-if="position === 'left'"
    :class="`question-mark ${additionalClasses}`"
    v-tooltip.left="text"
    v-once
  />
  <div
    v-else-if="position === 'right'"
    :class="`question-mark ${additionalClasses}`"
    v-tooltip.right="text"
    v-once
  />
  <div
    v-else-if="position === 'top'"
    :class="`question-mark ${additionalClasses}`"
    v-tooltip.top="text"
    v-once
  />
  <div
    v-else-if="position === 'bottom'"
    :class="`question-mark ${additionalClasses}`"
    v-tooltip.bottom="text"
    v-once
  />
</template>
