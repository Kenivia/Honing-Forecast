<script setup lang="ts">
import { computed, ref, watchPostEffect } from "vue";
import { OneIconConfig, SlotResult } from "./LoadStorage";
import { SlotEdit } from "./ScanStore";
import { draw_icon, status_border } from "./ScannerUIutils";
import Spinner from "@/Components/Common/Spinner.vue";

const props = defineProps<{
  info?: SlotResult;
  edit?: SlotEdit;
  image?: OneIconConfig;
  active: boolean;
}>();

const border = computed(() =>
  status_border(props.info?.status ?? "Pending", !!props.edit),
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
    class="relative aspect-square cursor-pointer"
    :class="{ 'z-10': active }"
    :style="{
      ...border,
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
    <Spinner
      v-if="!edit && info?.reading"
      class="absolute inset-[18.75%] h-[62.5%] w-[62.5%] text-(--text-bright)"
    />
  </div>
</template>
