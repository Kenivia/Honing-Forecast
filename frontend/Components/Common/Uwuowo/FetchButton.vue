<script setup lang="ts">
import {
  UwuowoPiece,
  UwuowoRegions,
  UwuowoResult,
  get_parsed_uwuowo,
} from "./UwuowoUtils";
import Popup from "../Popup.vue";
import { computed, Ref, ref, watch } from "vue";
import { PIECE_NAMES } from "@/Utils/Constants";
import LabeledPieceIcon from "../LabeledPieceIcon.vue";

const props = defineProps<{
  region: UwuowoRegions;
  char_name: string;
  apply?: (_: UwuowoPiece[], force_t4: boolean) => void;
}>();

watch([() => props.region, () => props.char_name], () => {
  parse_msg.value = fetch_msg;
  parsed_result.value = null;
});

const fetch_msg = "Fetching from lostark.bible...";
const parse_msg = ref(fetch_msg);
const parsed_result: Ref<null | UwuowoResult> = ref(null);
const show_popup = ref(false);

async function click() {
  show_popup.value = true;
  const result = await get_parsed_uwuowo(props.region, props.char_name);
  if (typeof result === "string") {
    parse_msg.value = result;
    parsed_result.value = null;
  } else {
    parse_msg.value = "";
    parsed_result.value = result;
  }
}

function confirm(force_t4: boolean) {
  show_popup.value = false;
  props.apply(parsed_result.value.pieces, force_t4);
}
const have_both = computed(
  () =>
    parsed_result.value &&
    parsed_result.value.pieces.some((x) => x.tier === 1) &&
    parsed_result.value.pieces.some(
      (x) => x.tier === 0 && (x.plus_n < 20 || x.adv < 40),
    ),
);
</script>

<template>
  <button @click="click" class="generic-button w-full text-wrap!">
    Fetch from lostark.bible
  </button>
  <Popup :show_popup="show_popup" @close_popup="show_popup = false">
    <span> {{ parse_msg }}</span>
    <div v-if="parsed_result" class="flex flex-col">
      <span class="text-2xl">This you?</span>
      <span>
        Character <span class="font-bold">{{ props.char_name }} </span> (region:
        {{ props.region }})</span
      >
      <span> Class: {{ parsed_result.class_name }} </span>
      <span class="pb-2 text-(--achieved)">
        Achieved ilevel: {{ parsed_result.achieved_ilevel }}</span
      >
      <div class="grid grid-cols-[2fr_1fr_1fr_1fr]">
        <div
          v-for="(piece, index) in PIECE_NAMES"
          :key="piece"
          class="contents"
        >
          <LabeledPieceIcon :piece="piece" />
          <span class="px-1">
            {{ parsed_result.pieces[index].tier === 0 ? "T4" : "Serca" }}</span
          >
          <span class="px-1"> +{{ parsed_result.pieces[index].plus_n }}</span>
          <span class="px-1">{{ parsed_result.pieces[index].ilevel }}</span>
        </div>
      </div>
    </div>
    <a
      :href="`https://lostark.bible/character/${region}/${char_name}`"
      class="link"
      v-if="!parsed_result && parse_msg !== fetch_msg"
    >
      {{ `https://lostark.bible/character/${region}/${char_name}` }}</a
    >

    <div v-if="have_both" class="flex w-72 flex-col">
      <span>
        You seem to have some Serca gear AND some T4 gear that's not ready to
        transfer. We cannot calculate the cost of upgrades from different tiers
        at the same time.
      </span>
      <span> Pick what tier you want to calculate: </span>
    </div>
    <div class="flex w-full flex-row justify-around">
      <button
        @click="show_popup = false"
        class="generic-button bg-(--warning-dark)! text-(--text-muted)!"
      >
        Cancel
      </button>

      <button
        class="generic-button text-(--achieved)!"
        v-if="parsed_result && parse_msg !== fetch_msg && have_both"
        @click="() => confirm(true)"
      >
        Confirm {{ have_both ? "(T4)" : "" }}
      </button>
      <button
        class="generic-button text-(--achieved)!"
        v-if="parsed_result && parse_msg !== fetch_msg"
        @click="() => confirm(false)"
      >
        Confirm {{ have_both ? "(Serca)" : "" }}
      </button>
    </div>
  </Popup>
</template>
