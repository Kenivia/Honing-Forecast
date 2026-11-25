<script setup lang="ts">
import NameInput from "../NameInput.vue";
import RegionSelector from "../RegionSelector.vue";
import FetchCharButton from "./FetchCharButton.vue";
import { MarketRegions } from "@/Utils/MarketDataFetcher.js";
import { ApplyFetch, market_to_uwuowo_region } from "./UwuowoUtils.js";
import { computed, ref, watch } from "vue";
import { CharProfile } from "@/Stores/CharacterProfile";
import { useRosterStore } from "@/Stores/RosterConfig";

const props = defineProps<{
  profile: CharProfile;
  region: MarketRegions; // this is needed even if region is hidden (for the fetch)
  hide_region?: boolean;
  region_change?: (_event: any) => void;
  apply?: ApplyFetch;
  card?: boolean; // roster card: no label and a bigger name once one is typed, refresh shown by the card
}>();

const roster_store = useRosterStore();
const counter = ref(0);
const taken = computed(() => roster_store.name_taken(props.profile));

function rename(typed: string) {
  roster_store.rename_char(props.profile.id, typed);
  draft.value = props.profile.char_name;
  counter.value += 1;
}

// what is in the box, typed or saved; the card drops its label as soon as it holds anything
const draft = ref(props.profile.char_name);
watch(
  () => props.profile.char_name,
  (name) => (draft.value = name),
);
const named = computed(() => props.card && draft.value !== "");

// signed in and the character is in a linked roster: name and region locked, refresh instead of the popup
const linked = computed(() => roster_store.char_linked(props.profile));
</script>
<template>
  <div v-if="taken" class="basis-full pl-1 text-sm text-(--warning)">
    {{ profile.char_name }} already exists
  </div>
  <div
    class="flex h-fit basis-full flex-row items-center gap-1 py-0.5 pl-1"
    :class="card ? 'w-full' : 'w-min justify-around'"
  >
    <span v-if="!named" class="w-fit text-nowrap"> Char name:</span>
    <NameInput
      :name="profile.char_name"
      :placeholder="roster_store.char_default_label(profile)"
      label="Char name"
      :locked="linked"
      :class="[
        card ? 'min-w-0 flex-1' : 'min-w-full',
        { 'text-xl!': named, 'pl-0!': card && linked },
      ]"
      @input="draft = ($event.target as HTMLInputElement).value"
      @rename="rename"
    />
  </div>
  <div
    class="flex basis-full flex-row items-center gap-2"
    :class="{ 'pl-1': card }"
  >
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
      :hide_refresh="card"
    />
  </div>
</template>
