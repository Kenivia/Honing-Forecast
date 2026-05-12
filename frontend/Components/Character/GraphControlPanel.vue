<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { BUTTON_LABELS, CSS_NAMES } from "@/Utils/Constants";

const store = useRosterStore();
const { roster_config, enabled_annotations } = storeToRefs(store);

const GRAPH_COLOR_VARS = ["--average", "--bound", "--roster", "--tradable"];
</script>

<template>
  <section class="w-full items-center">
    <div class="control-panel-title">Graph options</div>

    <div class="gap-1 px-0 text-sm">
      <label class="control-panel-checkbox-row">
        <input v-model="roster_config.cumulative_graph" type="checkbox" />
        <span>Cumulative graph</span>
      </label>
      <button
        v-for="(label, index) in BUTTON_LABELS"
        :key="index"
        class="ml-3 cursor-pointer rounded-full px-1 text-xs"
        :style="{
          border: `1px solid var(${enabled_annotations[index] ? '--border-main' : '--border-muted'})`,
          background: enabled_annotations[index]
            ? `var(${GRAPH_COLOR_VARS[index]})`
            : '',
          color: `var(${enabled_annotations[index] ? '--text-void' : '--text-main'})`,
        }"
        @click="enabled_annotations[index] = !enabled_annotations[index]"
      >
        {{ label }}
      </button>
    </div>
  </section>
</template>
