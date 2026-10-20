<script setup lang="ts">
import { computed, ref } from "vue";
import { getModel, getScannerConfig, OneIconConfig } from "./LoadStorage.js";
import { process_result } from "./ScanStore";
import { SHARED_ICONS } from "./Manifest";
import CurrencyBar from "./CurrencyBar.vue";
import Manifest from "./Manifest.vue";
import SlotGrid from "./SlotGrid.vue";
import Stream from "./Stream.vue";

const config = ref<OneIconConfig[] | null>(null);
getScannerConfig().then((data) => (config.value = data));
// fetched on the first visit, not on app load
getModel();

const status = ref<"idle" | "capturing">("idle");
// what a slot can be set to by hand
const items = computed(() =>
  (config.value ?? [])
    .filter((icon) => icon.tag === "Icon")
    .flatMap((icon) => SHARED_ICONS[icon.name] ?? [icon.name])
    .sort(),
);
</script>

<template>
  <div v-if="config" class="flex w-full flex-col gap-3">
    <Stream v-model:status="status" :process_result="process_result" />
    <CurrencyBar />
    <SlotGrid :items="items" />
    <Manifest />
  </div>
</template>
