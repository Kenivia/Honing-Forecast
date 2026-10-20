<script setup lang="ts">
export interface Props {
  text: string;
  position?: "left" | "right" | "top" | "bottom" | "none";
}

withDefaults(defineProps<Props>(), {
  position: "none",
});
</script>

<template>
  <div v-if="position === 'none'" class="question-mark" v-tooltip="text" />
  <div
    v-else-if="position === 'left'"
    class="question-mark"
    v-tooltip.left="text"
  />
  <div
    v-else-if="position === 'right'"
    class="question-mark"
    v-tooltip.right="text"
  />
  <div
    v-else-if="position === 'top'"
    class="question-mark"
    v-tooltip.top="text"
  />
  <div
    v-else-if="position === 'bottom'"
    class="question-mark"
    v-tooltip.bottom="text"
  />
</template>
