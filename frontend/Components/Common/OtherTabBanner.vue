<script setup lang="ts">
import { other_tabs } from "@/Utils/OtherTabs";
</script>

<template>
  <div
    v-if="other_tabs.length"
    role="alert"
    class="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-(--warning-dark) bg-(--bg-muted) px-4 py-2 text-sm"
  >
    <span class="text-(--warning)">
      Honing Forecast is open in another tab. Each tab saves over the other's
      changes, so use one tab at a time.
    </span>
  </div>
</template>
