<script setup lang="ts">
import { WasmOp } from "@/WasmInterface/WasmWorker";
import { ref, watchEffect, toRaw } from "vue";
import {
  download_as_msg_pack,
  getScannerConfig,
  OneIconConfig,
  ScaledPosition,
} from "../LoadStorage";
import SetupIconRow from "./SetupIconRow.vue";

import { useRuntimeStore } from "@/Stores/RuntimeState";

defineProps<{ status: "idle" | "capturing" }>();
const boxes = defineModel<ScaledPosition[]>("boxes", { default: () => [] });

const runtime = useRuntimeStore();

const config = ref<OneIconConfig[] | null>(null);
getScannerConfig().then((data) => (config.value = data));

const setup_name = ref("");
const setup_tag = ref("");
const setup_brightness = ref(50);
const top_left_x = ref(0);
const top_left_y = ref(0);
const width = ref(100);
const height = ref(100);

watchEffect(() => {
  const x = Number(top_left_x.value) || 0;
  const y = Number(top_left_y.value) || 0;
  boxes.value = [
    {
      top_left: [x, y],
      width: Number(width.value) || 0,
      height: Number(height.value) || 0,
    },
  ];
});

async function confirm_setup() {
  const bundle = runtime.cropper;
  if (!bundle?.result) return;
  const frame = await runtime.frame_source.read();
  if (!frame) return;

  const incoming_new_icons = [
    {
      position: {
        top_left: [Number(top_left_x.value), Number(top_left_y.value)],
        width: Number(width.value) || 0,
        height: Number(height.value) || 0,
      },
      name: setup_name.value,
      tag: setup_tag.value,
      brightness: setup_brightness.value,
    },
  ];

  bundle.debounced_start(
    WasmOp.Setup,
    { config: toRaw(config.value), incoming_new_icons, frame },
    (result) => {
      config.value = result.config;
    },
    0,
    false,
  );
}

function download_config() {
  download_as_msg_pack(config.value, "ScannerConfig.msgpack");
}

function delete_icon(index: number) {
  if (!config.value) return;
  config.value.splice(index, 1);
}

function move_icon_up(index: number) {
  if (!config.value || index <= 0) return;
  [config.value[index - 1], config.value[index]] = [
    config.value[index],
    config.value[index - 1],
  ];
}

function move_icon_down(index: number) {
  if (!config.value || index >= config.value.length - 1) return;
  [config.value[index], config.value[index + 1]] = [
    config.value[index + 1],
    config.value[index],
  ];
}

function update_icon_name(index: number, name: string) {
  if (!config.value) return;
  config.value[index].name = name;
}

function update_icon_tag(index: number, tag: string) {
  if (!config.value) return;
  config.value[index].tag = tag;
}

function update_icon_position(index: number, position: ScaledPosition) {
  if (!config.value) return;
  config.value[index].offset = position;
}

function update_icon_required_confidence(index: number, value: number | null) {
  if (!config.value) return;
  config.value[index].required_confidence = value;
}

function import_position(icon: OneIconConfig) {
  setup_name.value = icon.name;
  setup_tag.value = icon.tag;
  setup_brightness.value = 50;
  top_left_x.value = icon.offset.top_left[0];
  top_left_y.value = icon.offset.top_left[1];
  width.value = icon.offset.width;
  height.value = icon.offset.height;
}
</script>

<template>
  <div class="card-shell card-body mt-4 flex flex-col gap-3">
    <span class="text-sm font-semibold">New Icon Setup</span>

    <div class="flex flex-wrap items-end gap-3">
      <label class="flex flex-col gap-1 text-xs">
        Top left X
        <input
          v-model.number="top_left_x"
          type="number"
          class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        Top left Y
        <input
          v-model.number="top_left_y"
          type="number"
          class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        Width
        <input
          v-model.number="width"
          type="number"
          min="0"
          class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        Height
        <input
          v-model.number="height"
          type="number"
          min="0"
          class="w-24 rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        Name
        <input
          v-model="setup_name"
          type="text"
          class="rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        Tag
        <input
          v-model="setup_tag"
          type="text"
          class="rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        brightness
        <input
          v-model="setup_brightness"
          type="number"
          class="rounded bg-zinc-800 px-2 py-1 text-sm"
        />
      </label>
      <button
        class="rounded-md bg-blue-600 px-3 py-1.5 text-sm transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="!setup_name || status !== 'capturing'"
        @click="confirm_setup"
      >
        Confirm Icon
      </button>

      <button
        class="rounded-md bg-blue-600 px-3 py-1.5 text-sm transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40"
        @click="download_config"
      >
        Download file
      </button>
    </div>
  </div>

  <div class="mt-4 flex flex-col gap-2">
    <SetupIconRow
      v-for="(icon, index) in config"
      :key="icon.name + index"
      :icon="icon"
      :can-move-up="index > 0"
      :can-move-down="index < config.length - 1"
      @update:name="(value) => update_icon_name(index, value)"
      @update:tag="(value) => update_icon_tag(index, value)"
      @update:position="(value) => update_icon_position(index, value)"
      @update:required-confidence="
        (value) => update_icon_required_confidence(index, value)
      "
      @import-position="(value) => import_position(value)"
      @move-up="move_icon_up(index)"
      @move-down="move_icon_down(index)"
      @delete="delete_icon(index)"
    />
  </div>
</template>
