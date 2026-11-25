<script setup lang="ts">
// The small square .icon-button: a public icon in the text colour, or the slot instead
import { icon_mask } from "@/Utils/Helpers";

defineProps<{ icon?: string }>();
</script>

<template>
  <button class="icon-button">
    <slot>
      <span class="h-full w-full bg-current" :style="icon_mask(icon)" />
    </slot>
  </button>
</template>
