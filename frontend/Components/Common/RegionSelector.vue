<script setup lang="ts">
import { MarketRegions } from "@/Utils/MarketDataFetcher";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();

defineProps<{
  region: MarketRegions;
  region_change: (event) => void;
  locked?: boolean; // linked to lostark.bible: shown as text
}>();
</script>
<template>
  <div class="flex max-w-fit flex-row items-center gap-2">
    <span> Region: </span>
    <select
      :value="region"
      @change="region_change"
      class="selector"
      :class="{
        'cursor-default appearance-none border-transparent! text-inherit opacity-100':
          locked,
      }"
      aria-label="Region"
      :disabled="locked || runtime.is_fetching[region]"
    >
      <option value="nae">NA</option>
      <option value="euc">EU</option>
      <option value="Custom">Custom</option>
    </select>
  </div>
</template>
