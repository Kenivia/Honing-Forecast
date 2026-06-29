<script setup lang="ts">
import {
  DEFAULT_UWUOWO_BUNDLE,
  FETCH_MSG,
  UwuowoPiece,
  UwuowoRegions,
  fetch_and_parse,
  parse_char,
  reset_uwuowo_bundle,
} from "./UwuowoUtils";
import Popup from "../Popup.vue";
import ConfirmPanel from "./ConfirmPanel.vue";
import { ref, watch } from "vue";

const props = defineProps<{
  region: UwuowoRegions;
  char_name: string;
  apply: (_: UwuowoPiece[], force_t4: boolean) => void;
  counter?: number; // for triggering when char name input changes
}>();

watch(
  () => props.counter,
  () => {
    click();
  },
);

const char_parsed = ref(structuredClone(DEFAULT_UWUOWO_BUNDLE));

// const roster_
watch([() => props.region, () => props.char_name], () => {
  reset_uwuowo_bundle(char_parsed);
});

const show_popup = ref(false);

async function click() {
  // console.log("click triggered");
  show_popup.value = true;
  const char_result = await fetch_and_parse(
    {
      region: props.region,
      char_name: props.char_name,
      suffix: "",
    },
    parse_char,
  );
  // console.log(result, props.region, props.char_name);
  if (typeof char_result === "string") {
    char_parsed.value.status = char_result;
    char_parsed.value.result = null;
  } else {
    char_parsed.value.status = "";
    char_parsed.value.result = char_result;
  }
}
</script>

<template>
  <button
    @click="click"
    class="generic-button w-full text-wrap!"
    :disabled="
      // true
      region === null
    "
    :style="{
      opacity: region === null ? 0.5 : 1,
      cursor: region === null ? 'not-allowed' : 'pointer',
    }"
  >
    Fetch from lostark.bible
  </button>
  <Popup :show_popup="show_popup" @close_popup="show_popup = false">
    <span> {{ char_parsed.status }}</span>
    <span class="text-2xl" v-if="char_parsed.result !== null">This you?</span>
    <ConfirmPanel
      :region="region"
      :char_name="char_name"
      :apply="apply"
      :char_parsed="char_parsed"
      :close_on_success="true"
      @show_popup="
        (new_state) => {
          show_popup = new_state;
        }
      "
    />
  </Popup>
</template>
