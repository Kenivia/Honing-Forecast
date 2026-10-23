<script setup lang="ts">
import { assume_char_bound, assume_roster_tradable } from "./ScanStore";
import { show_debug } from "./ScannerUIutils";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";

const { roster_config } = storeToRefs(useRosterStore());

const analytics_tooltip = `Sends your ilevel and materials and chests owned amounts to help estimate how much players earn (for a future income forecasting feature). Only the numbers are sent, not captured footage`;
</script>
<template>
  <div class="w-full items-center">
    <div class="control-panel-title">Scanner</div>

    <div class="py-1 text-sm">
      <label class="control-panel-checkbox-row">
        <input v-model="assume_roster_tradable" type="checkbox" />
        <span>Assume roster storage is tradable</span>
      </label>
      <label class="control-panel-checkbox-row">
        <input v-model="assume_char_bound" type="checkbox" />
        <span>Assume char storage and inventory are char-bound</span>
      </label>
      <label class="control-panel-checkbox-row">
        <input v-model="show_debug" type="checkbox" />
        <span>Show debug options</span>
      </label>
      <div class="control-panel-checkbox-row">
        <label class="flex flex-row items-center gap-2">
          <input v-model="roster_config.analytics" type="checkbox" />
          <span>Share owned material anonymously</span>
        </label>
        <QuestionMark :text="analytics_tooltip" />
      </div>
    </div>
  </div>
</template>
