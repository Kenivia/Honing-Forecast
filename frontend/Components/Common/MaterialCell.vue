<script setup lang="ts">
import { cssVar, get_icon_path, locale_to_fixed } from "@/Utils/Helpers";
import { get_modified_cell, InputColumn } from "@/Utils/InputColumn";

import { computed, ref, watch } from "vue";

const props = defineProps<{
  input_column: InputColumn | number[];
  row: number;
  label?: string;
  setter?: (val: string) => void;
  suffix?: string;
  input_color?: string;
  is_percentage?: boolean;
  hide_tick?: boolean;
  treat_as_two?: boolean;
  callback?: () => void;
  input_width?: number;
  label_width?: number;
  icon_size?: number;
  height?: number;
  hide_label?: boolean;
  justify_left?: boolean;
  disabled?: boolean;
}>();

const actual_input_width = computed(() => `${props.input_width ?? 100}px`);
const actual_label_width = computed(() => `${props.label_width ?? 150}px`);
const actual_icon_size = computed(() => `${props.icon_size ?? 34}px`);
const actual_height = computed(() => `${props.height ?? 42}px`);
const resolved_color = computed(() =>
  cssVar(props.input_color, props.input_color),
);

const source_value = () =>
  String(
    !Array.isArray(props.input_column)
      ? (props.input_column as InputColumn).data[props.row]
      : props.input_column[props.row],
  );

const this_data = ref(source_value());

watch(source_value, (val) => {
  this_data.value = val;
});
</script>

<template>
  <div
    class="material-cell"
    :style="{
      gridColumn: treat_as_two ? 'span 2' : 'span 1',
      height: actual_height,
      justifyContent: justify_left ? 'left' : 'right',
    }"
  >
    <input
      v-if="!hide_tick && label && !Array.isArray(input_column)"
      type="checkbox"
      v-model="(input_column as InputColumn).enabled[row]"
      @change="callback"
    />
    <label
      v-if="label"
      class="row-label"
      :style="{
        width: actual_label_width,
      }"
    >
      <span v-if="!hide_label">{{ label }}</span>
      <img
        :src="get_icon_path(label)"
        class="generic-icon"
        :style="{
          width: actual_icon_size,
          height: actual_icon_size,
        }"
        :alt="label"
      />
    </label>

    <input
      v-if="!Array.isArray(input_column)"
      type="text"
      class="generic-input"
      :style="{
        color: resolved_color,
        width: actual_input_width,
        minWidth: actual_input_width,
      }"
      v-model="this_data"
      @change="
        ((this_data = get_modified_cell(input_column, row, $event)),
        setter(get_modified_cell(input_column, row, $event)),
        callback ? callback() : null)
      "
      :disabled="!input_column.enabled[row]"
    />
    <label
      v-else
      class="material-cell-result"
      :style="{ color: resolved_color }"
      type="text"
      >{{
        is_percentage
          ? locale_to_fixed(input_column[row] * 100, 2) + "%"
          : input_column[row].toLocaleString("en-US", {
              minimumFractionDigits: 0, // show decimals for small K/M/B
              maximumFractionDigits: 0,
            })
      }}</label
    >
      <label class="material-cell-suffix annotation" v-if="suffix">{{
        suffix
      }}</label>
  </div>
  <!-- <input v-if="customLeftovers" type="text" :value="matsLeftover[label]" @input="setRecordValue(matsLeftover, label, $event)" /> -->
</template>
<style scoped>
.material-cell {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 16px;
  min-width: 0;
  text-align: right;
  height: 36px; /* 100% seems to be a tiny bit off for some reason */
}

.material-cell-result {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  font-size: 16px;
  min-width: 0;
}

.material-cell-suffix {
  text-align: left;
  /* justify-self: right; */

  /* margin-left: auto;
  position: absolute;
  transform: translateX(100%);
  right: 50px; */
}
.row-label {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
  color: var(--text-muted);
  font-size: 13px;
  min-width: 0;
  text-align: right;
  padding-right: 8px;
}
</style>
