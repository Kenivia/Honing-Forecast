<script setup lang="ts">
import CharNameInput from "./CharNameInput.vue";
import RegionSelector from "../RegionSelector.vue";
import FetchCharButton from "./FetchCharButton.vue";
import { MarketRegions } from "@/Utils/MarketDataFetcher.js";
import { ApplyFetch, market_to_uwuowo_region } from "./UwuowoUtils.js";
import { computed, ref } from "vue";
import { linked_now } from "@/Utils/BibleAuth";

const props = defineProps<{
  name: string;
  profile_index: number;
  name_change: (_: string) => void;
  region: MarketRegions; // this is needed even if region is hidden (for the fetch)
  hide_region?: boolean;
  region_change?: (_event: any) => void;
  apply?: ApplyFetch;
  profile_id?: string;
}>();

const counter = ref(0);

// signed in and the character is in a linked roster: name and region locked, refresh instead of the popup
const linked = computed(() => {
  const region = market_to_uwuowo_region(props.region);
  return (
    props.profile_id !== undefined &&
    region !== null &&
    linked_now(region, props.name) !== null
  );
});
</script>
<template>
  <CharNameInput
    :char_name="name"
    :profile_index="profile_index"
    :locked="linked"
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
      :locked="linked"
      v-if="!hide_region"
    />
    <FetchCharButton
      :char_name="name"
      :region="market_to_uwuowo_region(region)"
      :apply="apply"
      :counter="counter"
      :profile_id="profile_id"
      :linked="linked"
    />
  </div>
</template>
