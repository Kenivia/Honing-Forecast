<script setup lang="ts">
import { CharProfile } from "@/Stores/CharacterProfile";
import { UwuowoRegions, UwuowoResult, get_parsed_uwuowo } from "./UwuowoUtils";
import Popup from "../Popup.vue";
import { Ref, ref } from "vue";
import { PIECE_NAMES } from "@/Utils/Constants";

const props = defineProps<{
  region: UwuowoRegions;
  char_name: string;
}>();

const emit = defineEmits<{
  apply: [];
}>();

const parse_msg = ref("Fetching from lostark.bible...");
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

function confirm() {
  show_popup.value = false;
  emit("apply");
}
</script>

<template>
  <button @click="click" class="generic-button">fetch from uwuowo</button>
  <Popup :show_popup="show_popup" @close_popup="show_popup = false">
    <span> {{ parse_msg }}</span>
    <div v-if="parsed_result">
      <span>{{ parsed_result.class_name }} </span>
      <span v-for="(piece, index) in PIECE_NAMES" :key="piece">
        {{ piece }} +{{ parsed_result.pieces[index][0] }}
        {{ parsed_result.pieces[index][1] }}</span
      >
    </div>

    <button @click="confirm">Confirm</button>
  </Popup>
</template>
