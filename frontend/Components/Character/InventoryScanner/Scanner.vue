<script setup lang="ts">
import { computed, ref } from "vue";
import { getScannerConfig, OneIconConfig } from "./LoadStorage.js";
import { process_result } from "./ScanStore";
import { SHARED_ICONS } from "./Manifest";
import Manifest from "./Manifest.vue";
import SlotGrid from "./SlotGrid.vue";
import Stream from "./Stream.vue";

const config = ref<OneIconConfig[] | null>(null);
getScannerConfig().then((data) => (config.value = data));

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
    <SlotGrid :items="items" />
    <Manifest />
  </div>
</template>
