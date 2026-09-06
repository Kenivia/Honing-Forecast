<script setup lang="ts">
import { ref } from "vue";
import SetupInterface from "./SetupInterface.vue";
import {
  getScannerConfig,
  OneIconConfig,
  ScaledPosition,
} from "../LoadStorage.js";
import Stream from "../Stream.vue";

const config = ref<OneIconConfig[] | null>(null);
getScannerConfig().then((data) => (config.value = data));

const status = ref<"idle" | "capturing">("idle");
const boxes = ref<ScaledPosition[]>([]);
</script>

<template>
  <div v-if="config">
    <Stream
      :should_start_cropper="false"
      :debugging="true"
      v-model:status="status"
      :boxes="boxes"
    />
    <SetupInterface :status="status" v-model:boxes="boxes" />
  </div>

  <!-- table of observed icons -->
</template>
