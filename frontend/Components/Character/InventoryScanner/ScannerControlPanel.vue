<script setup lang="ts">
import { assume_char_bound, assume_roster_tradable } from "./ScanStore";
import { show_debug } from "./ScannerUIutils";
</script>
<template>
  <div class="w-full items-center">
    <div class="control-panel-title">Scanner</div>

    <div class="py-1 text-sm">
      <label class="control-panel-checkbox-row">
        <input v-model="assume_roster_tradable" type="checkbox" />
        <span>Assume roster storage is tradable</span>
      </label>
      <label class="control-panel-checkbox-row">
        <input v-model="assume_char_bound" type="checkbox" />
        <span>Assume char storage and inventory are char-bound</span>
      </label>
      <label class="control-panel-checkbox-row">
        <input v-model="show_debug" type="checkbox" />
        <span>Show debug options</span>
      </label>
    </div>
  </div>
</template>
