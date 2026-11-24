<script setup lang="ts">
import NameInput from "../NameInput.vue";
import RegionSelector from "../RegionSelector.vue";
import FetchCharButton from "./FetchCharButton.vue";
import { MarketRegions } from "@/Utils/MarketDataFetcher.js";
import { ApplyFetch, market_to_uwuowo_region } from "./UwuowoUtils.js";
import { computed, ref } from "vue";
import { linked_now } from "@/Utils/BibleAuth";
import { CharProfile } from "@/Stores/CharacterProfile";
import { useRosterStore } from "@/Stores/RosterConfig";
import { name_problem } from "@/Utils/Helpers";

const props = defineProps<{
  profile: CharProfile;
  region: MarketRegions; // this is needed even if region is hidden (for the fetch)
  hide_region?: boolean;
  region_change?: (_event: any) => void;
  apply?: ApplyFetch;
}>();

const roster_store = useRosterStore();
const counter = ref(0);

// signed in and the character is in a linked roster: name and region locked, refresh instead of the popup
const linked = computed(() => {
  const region = market_to_uwuowo_region(props.region);
  return (
    region !== null && linked_now(region, props.profile.char_name) !== null
  );
});
</script>
<template>
  <div
    class="flex h-fit w-min basis-full flex-row justify-around gap-1 py-0.5 pl-1"
  >
    <span class="w-fit text-nowrap"> Char name:</span>
    <NameInput
      :name="profile.char_name"
      :placeholder="roster_store.char_default_label(profile)"
      label="Char name"
      :invalid="name_problem(profile.char_name) === 'Invalid name'"
      :locked="linked"
      class="min-w-full"
      @rename="
        (typed) => {
          roster_store.rename_char(profile.id, typed);
          counter += 1;
        }
      "
    />
  </div>
  <div class="flex basis-full flex-row items-center gap-2">
    <RegionSelector
      :region="region"
      :region_change="region_change"
      :locked="linked"
      v-if="!hide_region"
    />
    <FetchCharButton
      :char_name="profile.char_name"
      :region="market_to_uwuowo_region(region)"
      :apply="apply"
      :counter="counter"
      :profile_id="profile.id"
      :linked="linked"
    />
  </div>
</template>
