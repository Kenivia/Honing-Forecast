<script setup lang="ts">
import { ref } from "vue";
import SetupInterface from "./SetupInterface.vue";
import {
  getScannerConfig,
  OneIconConfig,
  ScaledPosition,
} from "./ScannerConfigStorage.js";

function start_cropper() {
  cropper_loop(cropper_worker_bundle.value.result);
}
async function cropper_loop(scanner_state: ScannerState) {
  if (
    cropper_worker_bundle.value === null ||
    cropper_worker_bundle.value.worker === null ||
    cropper_worker_bundle.value.result === null
  ) {
    console.log("no more cropper");
    cropper_running.value = false;
    return;
  }
  cropper_running.value = true;
  cropper_worker_bundle.value.debounced_start(
    WasmOp.Cropper,
    scanner_state,
    (scanner_state) => cropper_loop(scanner_state),
    0,
    false,
  );
}

import Stream from "../Stream.vue";
import { ScannerState } from "@/WasmInterface/WasmWorker.js";

const config = ref<OneIconConfig[] | null>(null);
getScannerConfig().then((data) => (config.value = data));

const status = ref<"idle" | "capturing">("idle");
const boxes = ref<ScaledPosition[]>([]);
</script>

<template>
  <div v-if="config">
    <Stream v-model:status="status" :boxes="boxes" />
    <SetupInterface v-model:boxes="boxes" :status="status" />
  </div>
</template>
