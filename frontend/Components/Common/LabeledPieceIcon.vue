<script setup lang="ts">
import { get_piece_bg, get_piece_icon } from "@/Utils/Helpers";

defineProps<{
  piece: string;
  class_name: string;
  tier: number;
  level: number;
}>();
</script>
<template>
  <div
    class="inline-flex w-full items-center justify-end gap-1.5 text-right text-sm text-(--text-muted)"
  >
    <span class="text-(--text-muted)">{{ piece }}</span>
    <img
      :src="get_piece_icon(piece, class_name, tier)"
      :alt="piece"
      :style="get_piece_bg(piece, level)"
      class="generic-icon piece-icon h-6.75 w-6.75"
    />
  </div>
</template>
