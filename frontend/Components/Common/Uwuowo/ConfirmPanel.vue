<script setup lang="ts">
import {
  FETCH_MSG,
  UwuowoResultBundle,
  ApplyFetch,
  UwuowoRegions,
} from "./UwuowoUtils";

import { computed } from "vue";
import { PIECE_NAMES } from "@/Utils/Constants";
import LabeledPieceIcon from "../LabeledPieceIcon.vue";

const props = defineProps<{
  region: UwuowoRegions;
  char_name: string;
  apply: ApplyFetch;
  char_parsed: UwuowoResultBundle;
  close_on_success: boolean;
}>();

const emit = defineEmits<{
  show_popup: [new_state: boolean];
}>();

function confirm() {
  if (props.close_on_success) {
    emit("show_popup", false);
  }

  const { pieces, class_name } = props.char_parsed.result;
  props.apply(pieces, props.char_name, class_name);
}

const link = computed(
  () => `https://lostark.bible/character/${props.region}/${props.char_name}`,
);
</script>

<template>
  <div class="w-60 rounded-sm border border-(--border-muted) px-3 py-2">
    <div v-if="char_parsed.result" class="flex flex-col">
      <span>
        <a class="link text-xl! font-bold break-all" :href="link"
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
          <LabeledPieceIcon
            :piece="piece"
            :class_name="char_parsed.result.class_name"
            :tier="char_parsed.result.pieces[index]?.tier ?? 0"
            :level="char_parsed.result.pieces[index]?.plus_n ?? 0"
            :sidereal_level="char_parsed.result.pieces[index]?.sidereal_level"
          />
          <div v-if="index < char_parsed.result.pieces.length" class="contents">
            <span class="px-1">
              {{
                char_parsed.result.pieces[index].sidereal_level !== undefined
                  ? "Sidereal"
                  : char_parsed.result.pieces[index].tier === 0
                    ? "T4"
                    : "Serca"
              }}</span
            >
            <span class="px-1">
              +{{
                char_parsed.result.pieces[index].sidereal_level ??
                char_parsed.result.pieces[index].plus_n
              }}</span
            >
            <span class="px-1">{{
              isNaN(char_parsed.result.pieces[index].ilevel)
                ? ""
                : char_parsed.result.pieces[index].ilevel
            }}</span>
          </div>
          <div v-else>
            <span class="px-1"> N/A</span>
          </div>
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

    <div class="flex w-full flex-row justify-around pt-2">
      <button
        v-if="char_parsed.result && char_parsed.status !== FETCH_MSG"
        class="generic-button text-(--achieved)!"
        @click="confirm"
      >
        Confirm
      </button>
      <button
        @click="emit('show_popup', false)"
        class="generic-button order-first bg-(--warning-dark)! text-(--text-bright)!"
      >
        Cancel
      </button>
    </div>
  </div>
</template>
