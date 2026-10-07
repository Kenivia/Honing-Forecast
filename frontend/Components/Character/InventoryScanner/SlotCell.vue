<script setup lang="ts">
import { computed, ref, watchPostEffect } from "vue";
import { OneIconConfig, SlotResult } from "./LoadStorage";
import { SlotEdit } from "./ScanStore";
import { draw_icon } from "./ScannerUIutils";

const props = defineProps<{
  info?: SlotResult;
  edit?: SlotEdit;
  image?: OneIconConfig;
  active: boolean;
}>();

const BORDERS = {
  Pending: "var(--text-muted)",
  Good: "var(--achieved)",
  NeedHover: "var(--series-fusion)",
  Error: "var(--warning)",
  Irrelevant: "var(--border-very-muted)",
};
const BORDER_WIDTHS = {
  Pending: "1px",
  Good: "1px",
  NeedHover: "3px",
  Error: "3px",
  Irrelevant: "1px",
};

const border = computed(() =>
  props.edit ? "var(--average)" : BORDERS[props.info?.status ?? "Pending"],
);

const border_width = computed(() =>
  props.edit ? "2px" : BORDER_WIDTHS[props.info?.status ?? "Pending"],
);

const canvas = ref<HTMLCanvasElement | null>(null);
watchPostEffect(() => {
  if (canvas.value && props.image) {
    const { width, height } = props.image.offset;
    draw_icon(canvas.value, props.image.data, width, height);
  }
});
</script>

<template>
  <div
    class="relative aspect-square cursor-pointer border"
    :class="{ 'border-dotted': edit, 'z-10': active }"
    :style="{
      borderColor: border,
      borderWidth: border_width,
      outline: active ? '3px dashed var(--text-bright)' : 'none',
    }"
  >
    <canvas
      v-if="image"
      ref="canvas"
      :width="image.offset.width"
      :height="image.offset.height"
      class="block h-full w-full"
      :class="{
        'opacity-30 grayscale': !edit && info?.status === 'Irrelevant',
      }"
    />
    <span
      v-else
      class="absolute inset-0 flex items-center justify-center text-(--text-muted)"
    >
      ?
    </span>
  </div>
</template>
