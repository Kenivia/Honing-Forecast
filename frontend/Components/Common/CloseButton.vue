<!-- square ✕ button, size in px; clicks and attrs fall through to the button -->
<script setup lang="ts">
const { size = 24 } = defineProps<{ size?: number }>();
</script>

<template>
  <button
    class="flex shrink-0 items-center justify-center rounded border border-(--border-main) text-(--text-muted) hover:text-(--text-bright)"
    :style="{ width: `${size}px`, height: `${size}px` }"
    aria-label="Close"
  >
    ✕
  </button>
</template>
