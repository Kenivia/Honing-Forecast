<script setup lang="ts">
import { get_icon_bg, get_icon_path } from "@/Utils/Helpers";

// "A and B", "A, B and C" with inline icons, each optionally with an amount (before the
// icon, or after it with icon_first)
defineProps<{
  items: { label: string; amount?: number }[];
  icon_first?: boolean;
}>();
</script>

<template>
  <template v-for="(item, i) in items" :key="item.label"
    >{{ i === 0 ? "" : i === items.length - 1 ? " and " : ", "
    }}<span
      class="inline-flex flex-row items-center gap-1 align-text-bottom whitespace-nowrap"
      :class="{ 'flex-row-reverse': icon_first }"
      ><b v-if="item.amount !== undefined">{{
        item.amount.toLocaleString("en-US")
      }}</b
      ><img
        :src="get_icon_path(item.label)"
        :style="get_icon_bg(item.label)"
        :alt="item.label"
        :title="item.label"
        class="piece-icon h-4.5 w-4.5" /></span
  ></template>
</template>
