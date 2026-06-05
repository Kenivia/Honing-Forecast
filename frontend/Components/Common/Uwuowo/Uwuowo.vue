<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import CharNameInput from "./CharNameInput.vue";
import RegionSelector from "../RegionSelector.vue";
import FetchButton from "./FetchButton.vue";
import { MarketRegions } from "@/Utils/MarketDataFetcher.js";
import { UwuowoPiece, UwuowoResult } from "./UwuowoUtils.js";

defineProps<{
  name: string;
  profile_index: number;
  name_change: (_: string) => void;
  region?: MarketRegions;
  hide_region?: boolean;
  region_change?: (_event: any) => void;
  apply?: (_: UwuowoPiece[]) => void;
}>();

const { active_profile } = storeToRefs(useRosterStore());
</script>
<template>
  <CharNameInput
    :char_name="name"
    :profile_index="profile_index"
    @char_name_change="name_change"
    class="flex-basis-full"
  />
  <div class="flex basis-full flex-row items-center gap-2">
    <RegionSelector
      :region="region"
      :region_change="region_change"
      v-if="!hide_region"
    />
    <FetchButton
      :char_name="active_profile.char_name"
      :region="region === 'nae' ? 'NA' : 'CE'"
      :apply="apply"
    />
  </div>
</template>
