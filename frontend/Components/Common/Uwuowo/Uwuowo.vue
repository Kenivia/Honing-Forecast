<script setup lang="ts">
import CharNameInput from "./CharNameInput.vue";
import RegionSelector from "../RegionSelector.vue";
import FetchButton from "./FetchButton.vue";
import { MarketRegions } from "@/Utils/MarketDataFetcher.js";
import { UwuowoPiece } from "./UwuowoUtils.js";
import { ref } from "vue";

defineProps<{
  name: string;
  profile_index: number;
  name_change: (_: string) => void;
  region: MarketRegions; // this is needed even if region is hidden (for the fetch)
  hide_region?: boolean;
  region_change?: (_event: any) => void;
  apply?: (_: UwuowoPiece[], force_t4?: boolean) => void;
}>();

const counter = ref(0);
</script>
<template>
  <CharNameInput
    :char_name="name"
    :profile_index="profile_index"
    @char_name_change="
      (new_name) => {
        name_change(new_name);
        counter += 1;

        // console.log(counter);
      }
    "
    class="flex-basis-full"
  />
  <div class="flex basis-full flex-row items-center gap-2">
    <RegionSelector
      :region="region"
      :region_change="region_change"
      v-if="!hide_region"
    />
    <FetchButton
      :char_name="name"
      :region="region === 'Custom' ? null : region === 'nae' ? 'NA' : 'CE'"
      :apply="apply"
      :counter="counter"
    />
  </div>
</template>
