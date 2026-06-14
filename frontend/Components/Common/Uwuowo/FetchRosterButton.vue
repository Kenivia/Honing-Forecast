<script setup lang="ts">
import {
  DEFAULT_UWUOWO_BUNDLE,
  FETCH_MSG,
  UwuowoPiece,
  UwuowoRegions,
  UwuowoResultBundle,
  fetch_and_parse,
  parse_char,
  parse_roster,
  reset_uwuowo_bundle,
} from "./UwuowoUtils";
import Popup from "../Popup.vue";
import { computed, ref, watch } from "vue";
import { PIECE_NAMES } from "@/Utils/Constants";
import LabeledPieceIcon from "../LabeledPieceIcon.vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import ConfirmPanel from "./ConfirmPanel.vue";

const props = defineProps<{
  region: UwuowoRegions;
  any_char_name: string;
  apply?: (_: UwuowoPiece[], force_t4: boolean, char_name?: string) => void;
}>();

const { all_profiles, roster_ids } = storeToRefs(useRosterStore());

const roster_parsed = ref(structuredClone(DEFAULT_UWUOWO_BUNDLE));

// const roster_
watch([() => props.region, () => props.any_char_name], () => {
  reset_uwuowo_bundle(roster_parsed);
});

const show_popup = ref(false);

const parsed_chars = ref<Record<string, UwuowoResultBundle>>({});
const fetching_chars = ref(false);
async function click() {
  // console.log("click triggered");
  show_popup.value = true;
  const roster_result = await fetch_and_parse(
    {
      region: props.region,
      char_name: props.any_char_name,
      suffix: "roster",
    },
    parse_roster,
  );
  // console.log(result, props.region, props.char_name);
  if (typeof roster_result === "string") {
    roster_parsed.value.status = roster_result;
    roster_parsed.value.result = null;
  } else {
    roster_parsed.value.status = "";
    roster_parsed.value.result = roster_result;
    parsed_chars.value = {};
    fetching_chars.value = true;
    for (const char of roster_parsed.value.result.filter((x) =>
      all_names.value.every((y) => y !== x),
    )) {
      parsed_chars.value[char] = {
        result: await fetch_and_parse(
          {
            region: props.region,
            char_name: char,
            suffix: "",
          },
          parse_char,
        ),
        status: "",
      };
      // doing these one by one to avoid like spamming uwuowo too fast
    }
    fetching_chars.value = false;
  }
  // console.log(parsed_chars.value);
}

const all_names = computed(() => all_profiles.value.map((x) => x.char_name));

const empty_parsed = computed(
  () =>
    roster_parsed.value.result !== null &&
    parsed_chars.value !== null &&
    !fetching_chars.value &&
    Object.entries(parsed_chars.value).length === 0,
);
const spread_multiple = computed(
  () =>
    empty_parsed.value &&
    new Set(
      roster_parsed.value.result.map(
        (char) =>
          roster_ids.value.findIndex(
            (x) =>
              x ===
              all_profiles.value.find((p) => p.char_name === char)?.roster_id,
          ) + 1,
      ),
    ).size > 1,
);
</script>

<template>
  <button
    @click="click"
    class="generic-button w-full text-wrap!"
    :disabled="region === null"
    :style="{
      opacity: region === null ? 0.5 : 1,
      cursor: region === null ? 'not-allowed' : 'pointer',
    }"
  >
    Fetch remaining characters
  </button>
  <Popup :show_popup="show_popup" @close_popup="show_popup = false">
    <span> {{ roster_parsed.status }}</span>
    <div v-if="empty_parsed" class="flex flex-col">
      <span class="pb-4"
        >All characters in
        <span class="text-lg font-bold">{{ any_char_name }}</span
        >'s roster are already
        {{ spread_multiple ? "somewhere in Honing Forecast." : "here." }}
      </span>
      <span v-if="spread_multiple"
        >Perhaps check if you put them the wrong roster?
      </span>
      <div v-if="spread_multiple" class="grid grid-cols-3 gap-2">
        <div v-for="char in roster_parsed.result" :key="char">
          <span
            >(Roster
            {{
              roster_ids.findIndex(
                (x) =>
                  x ===
                  all_profiles.find((p) => p.char_name === char)?.roster_id,
              ) + 1
            }}) </span
          >{{ char }}
        </div>
      </div>
      <button @click="show_popup = false" class="generic-button w-full px-20">
        Ok
      </button>
    </div>
    <div
      class="flex w-full flex-row flex-nowrap items-center justify-center gap-2 pb-2"
      v-else
    >
      <span class="text-center text-2xl">Are these your children?</span>
      <button
        class="generic-button w-fit! text-(--achieved)!"
        @click="
          () => {
            for (const char_name in parsed_chars) {
              apply(
                parsed_chars[char_name].result.pieces,
                false, // assume no forcing t4
                char_name,
              );
              delete parsed_chars[char_name];
            }
            show_popup = false;
          }
        "
      >
        Confirm all
      </button>
    </div>
    <div class="grid grid-cols-3 gap-20">
      <div
        v-for="(parsed_char, char_name) in parsed_chars"
        :key="char_name"
        class=""
      >
        <ConfirmPanel
          v-if="parsed_char !== undefined"
          :region="region"
          :char_name="char_name"
          :apply="
            (result, force_t4, char_name) => {
              delete parsed_chars[char_name];
              apply(result, force_t4, char_name);
            }
          "
          :char_parsed="parsed_char"
          :close_on_success="Object.keys(parsed_chars).length === 1"
          @show_popup="
            (new_state) => {
              show_popup = new_state;
            }
          "
        />
      </div>
      <span v-if="fetching_chars"> Fetching...</span>
    </div>
  </Popup>
</template>
