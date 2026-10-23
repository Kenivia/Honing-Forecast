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
import { scanner_shown } from "./ScannerUIutils";

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
if (scanner_shown.value) load();
function show_anyway() {
  scanner_shown.value = true;
  load();
}
// what a slot can be set to by hand
const items = computed(() =>
  (config.value ?? [])
    .filter((icon) => icon.tag === "Icon")
    .flatMap((icon) => SHARED_ICONS[icon.name] ?? [icon.name])
    .sort(),
);
</script>

<template>
  <div
    v-if="!scanner_shown"
    class="card-shell card-body flex items-center gap-3 text-sm"
  >
    <span class="text-(--text-main)">
      It looks like you're on mobile (or a browser that doesn't support screen
      sharing).
    </span>
    <button class="generic-button" @click="show_anyway">
      Show scanner anyway
    </button>
  </div>
  <div v-else-if="config" class="flex w-full flex-col items-center gap-3">
    <div class="flex max-w-full flex-wrap items-start justify-center gap-3">
      <Stream v-model:status="status" :process_result="process_result" />
      <ScanGuide />
    </div>
    <SlotGrid :items="items" />
    <Manifest />
  </div>
  <div v-else class="card-shell card-body flex items-center gap-3 text-sm">
    <template v-if="failed">
      <span class="text-(--warning)">
        Could not download the scanner's files. Check your connection.
      </span>
      <button
        class="generic-button bg-(--accent-blue)! hover:bg-(--accent-blue-hover)!"
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
