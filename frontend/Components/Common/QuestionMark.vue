<script setup lang="ts">
import HoverTip from "@/Components/Common/HoverTip.vue";

export interface Props {
  text: string;
  position?: "left" | "right" | "top" | "bottom" | "none";
}

withDefaults(defineProps<Props>(), {
  position: "none",
});
</script>

<template>
  <HoverTip
    class="question-mark"
    :text="text"
    :placement="position === 'none' ? 'bottom' : position"
  />
</template>
