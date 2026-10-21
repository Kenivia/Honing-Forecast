<script setup lang="ts">
import { computed, ref } from "vue";
import {
  download_note,
  getModel,
  getScannerConfig,
  OneIconConfig,
} from "./LoadStorage.js";
import { process_result } from "./ScanStore";
import { SHARED_ICONS } from "./Manifest";
import Manifest from "./Manifest.vue";
import ScanGuide from "./ScanGuide.vue";
import SlotGrid from "./SlotGrid.vue";
import Stream from "./Stream.vue";

const status = ref<"idle" | "capturing">("idle");
const config = ref<OneIconConfig[] | null>(null);
const failed = ref(false);
// fetched on the first visit, not on app load; a failed model is tried again when a capture starts
function load() {
  failed.value = false;
  getScannerConfig()
    .then((data) => (config.value = data))
    .catch(() => (failed.value = true));
  getModel().catch(() => {});
}
load();
// what a slot can be set to by hand
const items = computed(() =>
  (config.value ?? [])
    .filter((icon) => icon.tag === "Icon")
    .flatMap((icon) => SHARED_ICONS[icon.name] ?? [icon.name])
    .sort(),
);
</script>

<template>
  <div v-if="config" class="flex w-full flex-col items-center gap-3">
    <div class="flex max-w-full flex-wrap items-start justify-center gap-3">
      <Stream v-model:status="status" :process_result="process_result" />
      <ScanGuide />
    </div>
    <SlotGrid :items="items" />
    <Manifest />
  </div>
  <div v-else class="card-shell card-body flex items-center gap-3 text-sm">
    <template v-if="failed">
      <span class="text-red-400">
        Could not download the scanner's files. Check your connection.
      </span>
      <button
        class="rounded-md bg-blue-600 px-3 py-1.5 transition-colors hover:bg-blue-500"
        @click="load"
      >
        Try again
      </button>
    </template>
    <span v-else class="text-(--text-muted)">
      Downloading scanner data: {{ download_note ?? "starting" }}
    </span>
  </div>
</template>
