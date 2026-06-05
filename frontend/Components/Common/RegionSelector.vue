<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  fetch_callback,
  MarketRegions,
  useTimedFetch,
} from "@/Utils/MarketDataFetcher";
import { storeToRefs } from "pinia";

const roster_store = useRosterStore();
const { active_region } = storeToRefs(roster_store);

const { disabled, start_fetch } = useTimedFetch(fetch_callback);

function change_region(event) {
  roster_store.set_active_region(
    (event.target as HTMLSelectElement).value as MarketRegions,
  );
  console.log(active_region.value);
  start_fetch(active_region.value, true);
}
</script>
<template>
  <div class="flex flex-row gap-2 items-center">
    <span> Region: </span>
    <select
      :value="active_region"
      @change="change_region"
      class="selector"
      :disabled="disabled"
    >
      <option value="nae">NAE</option>
      <option value="euc">EUC</option>
    </select>
  </div>
</template>
