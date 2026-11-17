<script setup lang="ts">
import HoverTip from "@/Components/Common/HoverTip.vue";

// `text`, or the default slot for formatted content
export interface Props {
  text?: string;
  position?: "left" | "right" | "top" | "bottom" | "none";
}

withDefaults(defineProps<Props>(), {
  text: "",
  position: "none",
});
</script>

<template>
  <HoverTip
    class="question-mark"
    :text="text"
    :placement="position === 'none' ? 'right' : position"
  >
    <template v-if="$slots.default" #tip><slot /></template>
  </HoverTip>
</template>
