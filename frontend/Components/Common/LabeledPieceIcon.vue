<script setup lang="ts">
import { get_icon_path } from "@/Utils/Helpers";

defineProps<{ piece: string }>();
</script>
<template>
  <div
    class="inline-flex w-full items-center justify-end gap-1.5 text-right text-sm text-(--text-secondary)"
  >
    <span class="text-(--text-muted)">{{ piece }}</span>
    <img
      :src="get_icon_path(piece)"
      :alt="piece"
      class="generic-icon h-6.75 w-6.75"
    />
  </div>
</template>
