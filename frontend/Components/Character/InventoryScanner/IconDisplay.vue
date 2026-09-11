<script setup lang="ts">
import { ComponentPublicInstance } from "vue";
import { OneIconConfig } from "./LoadStorage";
import { draw_icon } from "./ScannerUIutils";

const props = defineProps<{
  icon: OneIconConfig;
}>();

function onCanvasRef(el: Element | ComponentPublicInstance | null) {
  if (el) {
    draw_icon(
      el as HTMLCanvasElement,
      props.icon.data,
      props.icon.offset.width,
      props.icon.offset.height,
    );
  }
}
</script>

<template>
  <canvas
    v-if="icon"
    :ref="onCanvasRef"
    :width="icon.offset.width"
    :height="icon.offset.height"
    class="rounded-none border-(--boredr-muted)"
    style="image-rendering: pixelated"
  />
  <div v-else>No Icon</div>
</template>
