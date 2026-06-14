<script setup lang="ts">
import {
  FETCH_MSG,
  UwuowoResultBundle,
  UwuowoPiece,
  UwuowoRegions,
} from "./UwuowoUtils";

import { computed } from "vue";
import { PIECE_NAMES } from "@/Utils/Constants";
import LabeledPieceIcon from "../LabeledPieceIcon.vue";

const props = defineProps<{
  region: UwuowoRegions;
  char_name: string;
  apply: (_: UwuowoPiece[], force_t4: boolean, char_name?: string) => void;
  char_parsed: UwuowoResultBundle;
  close_on_success: boolean;
}>();

const emit = defineEmits<{
  show_popup: [new_state: boolean];
}>();

function confirm(force_t4: boolean) {
  if (props.close_on_success) {
    emit("show_popup", false);
  }

  props.apply(props.char_parsed.result.pieces, force_t4, props.char_name);
}
const have_both = computed(() => {
  //   console.log(toRaw(props.char_parsed));
  return (
    props.char_parsed &&
    props.char_parsed.result &&
    props.char_parsed.result.pieces.some((x) => x.tier === 1) &&
    props.char_parsed.result.pieces.some(
      (x) => x.tier === 0 && (x.plus_n < 20 || x.adv < 40),
    )
  );
});

const link = computed(
  () => `https://lostark.bible/character/${props.region}/${props.char_name}`,
);
</script>

<template>
  <div class="rounded-sm border border-(--border-muted) px-3 py-2">
    <div v-if="char_parsed.result" class="flex flex-col">
      <span>
        <a class="link text-2xl! font-bold" :href="link"
          >{{ props.char_name }}
        </a></span
      >

      <span> Region: {{ props.region }}</span>
      <span> Class: {{ char_parsed.result.class_name }} </span>
      <span class="pb-2 text-(--achieved)">
        Achieved ilevel: {{ char_parsed.result.achieved_ilevel }}</span
      >
      <div class="-ml-3 grid grid-cols-[max-content_1fr_1fr_1fr]">
        <div
          v-for="(piece, index) in PIECE_NAMES"
          :key="piece"
          class="contents"
        >
          <LabeledPieceIcon :piece="piece" />
          <span class="px-1">
            {{
              char_parsed.result.pieces[index].tier === 0 ? "T4" : "Serca"
            }}</span
          >
          <span class="px-1">
            +{{ char_parsed.result.pieces[index].plus_n }}</span
          >
          <span class="px-1">{{
            char_parsed.result.pieces[index].ilevel
          }}</span>
        </div>
      </div>
    </div>
    <a
      :href="link"
      class="link"
      v-if="!char_parsed.result && char_parsed.status !== FETCH_MSG"
    >
      {{ link }}</a
    >

    <div v-if="have_both" class="flex w-full flex-col flex-wrap">
      <span>
        You seem to have some Serca gear AND some T4 gear that's not ready to
        transfer. We cannot calculate the cost of upgrades from different tiers
        at the same time.
      </span>
      <span> Pick what tier you want to calculate: </span>
    </div>
    <div v-if="!have_both" class="flex w-full flex-row justify-around pt-2">
      <button
        @click="emit('show_popup', false)"
        class="generic-button bg-(--warning-dark)! text-(--text-bright)!"
      >
        Cancel
      </button>

      <button
        class="generic-button text-(--achieved)!"
        v-if="
          char_parsed.result && char_parsed.status !== FETCH_MSG && have_both
        "
        @click="() => confirm(true)"
      >
        Confirm {{ have_both ? "(T4)" : "" }}
      </button>
      <button
        class="generic-button text-(--achieved)!"
        v-if="char_parsed.result && char_parsed.status !== FETCH_MSG"
        @click="() => confirm(false)"
      >
        Confirm {{ have_both ? "(Serca)" : "" }}
      </button>
    </div>
    <div v-else>
      <div class="flex flex-col items-center">
        <button
          class="generic-button text-(--achieved)!"
          v-if="
            char_parsed.result && char_parsed.status !== FETCH_MSG && have_both
          "
          @click="() => confirm(true)"
        >
          Confirm {{ have_both ? "(T4)" : "" }}
        </button>
        <div class="flex flex-row content-end">
          <button
            class="generic-button text-(--achieved)!"
            v-if="char_parsed.result && char_parsed.status !== FETCH_MSG"
            @click="() => confirm(false)"
          >
            Confirm {{ have_both ? "(Serca)" : "" }}</button
          ><span
            class="w-0 basis-0 origin-right transform-[translateY(8px)] pl-px text-left text-xs text-(--text-very-muted)"
            >(default)</span
          >
        </div>
        <button
          @click="emit('show_popup', false)"
          class="generic-button bg-(--warning-dark)! text-(--text-bright)!"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
</template>
