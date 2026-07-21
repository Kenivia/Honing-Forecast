<script setup lang="ts">
import { ref } from "vue";
import SetupInterface from "./SetupInterface.vue";
import {
  getScannerConfig,
  OneIconConfig,
  ScaledPosition,
} from "../ScannerConfigStorage.js";
import Stream from "../Stream.vue";

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
