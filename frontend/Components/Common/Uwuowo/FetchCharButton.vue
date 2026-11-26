<script setup lang="ts">
import {
  DEFAULT_UWUOWO_BUNDLE,
  ApplyFetch,
  UwuowoRegions,
  fetch_char,
  reset_uwuowo_bundle,
} from "./UwuowoUtils";
import Popup from "../Popup.vue";
import RefreshButton from "./RefreshButton.vue";
import ConfirmPanel from "./ConfirmPanel.vue";
import { computed, ref, watch } from "vue";
import { useTimeAgo } from "@vueuse/core";
import { find_linked, signed_in } from "@/Utils/BibleAuth";
import { refresh_char, refresh_errors } from "@/Utils/BibleSync";
import { useRosterStore } from "@/Stores/RosterConfig";
import { name_invalid, name_problem } from "@/Utils/Helpers";

const props = defineProps<{
  region: UwuowoRegions;
  char_name: string;
  apply: ApplyFetch;
  counter?: number; // for triggering when char name input changes
  profile_id?: string;
  linked?: boolean; // last fetched + refresh instead of the popup
  small?: boolean; // roster card: smaller last fetched line
}>();

const profile = computed(() =>
  useRosterStore().roster_config.profiles.find(
    (p) => p.id === props.profile_id,
  ),
);
const time_ago = useTimeAgo(() => profile.value?.fetched_at ?? 0);
const problem = computed(() =>
  props.char_name === ""
    ? "Enter a name to fetch"
    : name_problem(props.char_name),
);
const blocked = computed(() => props.region === null || !!problem.value);

watch(
  () => props.counter,
  async () => {
    if (problem.value) return;
    if (
      props.profile_id &&
      signed_in.value &&
      props.region !== null &&
      (await find_linked(props.region, props.char_name))
    )
      refresh_char(props.profile_id);
    else click();
  },
);

const char_parsed = ref(structuredClone(DEFAULT_UWUOWO_BUNDLE));

watch([() => props.region, () => props.char_name], () => {
  reset_uwuowo_bundle(char_parsed);
});

const show_popup = ref(false);

async function click() {
  const { region, char_name } = props;
  if (blocked.value) return;
  show_popup.value = true;
  const char_result = await fetch_char(region, char_name);
  // dropped if the name or region changed meanwhile
  if (props.region !== region || props.char_name !== char_name) return;
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
  <div v-if="linked" class="flex w-full flex-col">
    <div class="flex flex-row items-center gap-2">
      <span
        class="text-nowrap text-(--text-muted)"
        :class="{ 'text-sm': small }"
        >Last fetched {{ profile?.fetched_at ? time_ago : "never" }}</span
      >
      <RefreshButton :profile_id="profile_id" />
    </div>
    <span v-if="refresh_errors[profile_id]" class="text-xs text-(--warning)">{{
      refresh_errors[profile_id]
    }}</span>
  </div>
  <button
    v-else
    @click="click"
    class="generic-button w-full text-wrap!"
    :class="{ 'text-(--warning)!': name_invalid(char_name) }"
    :disabled="blocked"
    :style="{
      opacity: blocked ? 0.5 : 1,
      cursor: blocked ? 'not-allowed' : 'pointer',
    }"
  >
    {{ problem ?? "Fetch from lostark.bible" }}
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
