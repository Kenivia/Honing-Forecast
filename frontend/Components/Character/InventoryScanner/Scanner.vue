<script setup lang="ts">
import { ref } from "vue";
import {
  getScannerConfig,
  OneIconConfig,
  ScaledPosition,
  ScannerState,
} from "./ScannerConfigStorage.js";
import Stream from "./Stream.vue";
// import { draw_icon } from "./ScannerUIutils.js";
import IconDisplay from "./IconDisplay.vue";

const config = ref<OneIconConfig[] | null>(null);
getScannerConfig().then((data) => (config.value = data));

const status = ref<"idle" | "capturing">("idle");
const boxes = ref<ScaledPosition[]>([]);

interface DebugRow {
  icon_name: string;
  x: number;
  y: number;
  confidence: number;
  brightness: number;
}

const debug_table = ref<DebugRow[]>([]);

const debugging = ref(true);

interface FoundIconRow {
  key: string;
  icon: OneIconConfig;
  confidence: number;
  amount: OneIconConfig;
}

const found_icons = ref<FoundIconRow[]>([]);

function process_result(scanner_state: ScannerState) {
  scanner_state.debugging = debugging.value;

  if (debugging.value) {
    const new_boxes: ScaledPosition[] = [];
    const new_debug_table: DebugRow[] = [];

    for (const [icon_name, [position, confidence, brightness]] of [
      ...scanner_state.debug_info,
    ].sort(([a], [b]) => a.localeCompare(b))) {
      new_boxes.push(position);
      new_debug_table.push({
        icon_name,
        x: position.top_left[0],
        y: position.top_left[1],
        confidence,
        brightness,
      });
    }

    boxes.value = new_boxes;
    debug_table.value = new_debug_table;
  } else {
    boxes.value = [];
    debug_table.value = [];
  }

  const new_found_icons: FoundIconRow[] = [];
  for (const [_, slot_info] of [...scanner_state.slot_infos]
    .filter(([_, x]) => x.icon_name_score !== undefined)
    .sort(([__, a], [_, b]) =>
      a.icon_name_score[0].localeCompare(b.icon_name_score[0]),
    )) {
      // const id = slot_info.observed_id
    new_found_icons.push({
      key: slot_info.icon_name_score[0],
      icon: slot_info.observed_icon,
      confidence: slot_info.icon_name_score[1],
      amount: slot_info.observed_number,
    });
  }
  found_icons.value = new_found_icons;

  console.log("slot infos", scanner_state.slot_infos, found_icons.value);
}
</script>

<template>
  <div v-if="config">
    <Stream
      :should_start_cropper="true"
      v-model:status="status"
      :boxes="boxes"
      :debugging="debugging"
      :process_result="process_result"
    />
    <button class="generic-button" @click="debugging = !debugging">
      {{ debugging ? "Hide" : "Show" }} debug info
    </button>
    <div v-if="debugging">
      <table>
        <thead>
          <tr>
            <th>Icon name</th>
            <th>X</th>
            <th>Y</th>
            <th>Confidence</th>
            <th>Brightness</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in debug_table" :key="row.icon_name">
            <td>{{ row.icon_name }}</td>
            <td>{{ row.x.toFixed(1) }}</td>
            <td>{{ row.y.toFixed(1) }}</td>
            <td>{{ row.confidence.toFixed(3) }}</td>
            <td>{{ row.brightness.toFixed(3) }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <table>
      <thead>
        <tr>
          <th>Icon</th>
          <th>Name</th>
          <th>Tag</th>
          <th>X</th>
          <th>Y</th>
          <th>Confidience</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in found_icons" :key="row.key">
          <td>
            <IconDisplay :icon="row.icon"  />
          </td>
          <td>
            <IconDisplay :icon="row.amount" />
          </td>
          <td>{{ row.icon.name }}</td>
          <td>{{ row.icon.tag }}</td>
          <td>{{ row.icon.offset.top_left[0].toFixed(1) }}</td>
          <td>{{ row.icon.offset.top_left[1].toFixed(1) }}</td>
          <td>{{ row.confidence.toFixed(3) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style>
table {
  border-collapse: separate;
  border-spacing: 32px 0;
}
</style>
