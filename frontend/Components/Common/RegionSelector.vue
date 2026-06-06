<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { MarketRegions } from "@/Utils/MarketDataFetcher";
import { storeToRefs } from "pinia";

defineProps<{
  region: MarketRegions;
  region_change: (event) => void;
}>();
const roster_store = useRosterStore();
const { roster_config } = storeToRefs(roster_store);
</script>
<template>
  <div class="flex max-w-fit flex-row items-center gap-2">
    <span> Region: </span>
    <select
      :value="region"
      @change="region_change"
      class="selector"
      :disabled="roster_config.is_fetching"
    >
      <option value="nae">NA</option>
      <option value="euc">EU</option>
      <option value="Custom">Custom</option>
    </select>
  </div>
</template>
