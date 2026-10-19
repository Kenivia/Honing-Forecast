<script setup lang="ts">
import { MarketRegions } from "@/Utils/MarketDataFetcher";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();

defineProps<{
  region: MarketRegions;
  region_change: (event) => void;
}>();
</script>
<template>
  <div class="flex max-w-fit flex-row items-center gap-2">
    <span> Region: </span>
    <select
      :value="region"
      @change="region_change"
      class="selector"
      :disabled="runtime.is_fetching"
    >
      <option value="nae">NA</option>
      <option value="euc">EU</option>
      <option value="Custom">Custom</option>
    </select>
  </div>
</template>
