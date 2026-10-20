<script setup lang="ts">
import { computed } from "vue";
import type { Chest } from "@/Utils/Chest";

// A stack's art, its title on hover; a chest without known art shows its title.
const props = defineProps<{ chest: Chest }>();

const DIR = "../../../scanner_setup/templates/ChestIcons/";
const ART = import.meta.glob<string>(
  "../../../scanner_setup/templates/ChestIcons/*.png",
  { eager: true, query: "?url", import: "default" },
);
const srcs = computed(() =>
  props.chest.icons?.map((icon) => ART[`${DIR}${icon}.png`]),
);
</script>

<template>
  <span
    v-if="srcs?.length && srcs.every(Boolean)"
    class="flex items-center gap-0.5"
    :title="chest.title"
  >
    <img
      v-for="(src, i) in srcs"
      :key="i"
      :src="src"
      class="generic-icon h-8 w-8"
      :alt="chest.title"
    />
  </span>
  <span v-else>{{ chest.title }}</span>
</template>
