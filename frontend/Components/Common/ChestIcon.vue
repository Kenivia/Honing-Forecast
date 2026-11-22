<script setup lang="ts">
import { computed } from "vue";
import type { Chest } from "@/Utils/Chest";
import { rarity_bg } from "@/Utils/Helpers";

// A stack's art, its title on hover; a chest without known art shows its title.
const props = defineProps<{ chest: Chest }>();

const DIR = "../../../scanner_setup/templates/ChestIcons/";
const ART = import.meta.glob<string>(
  "../../../scanner_setup/templates/ChestIcons/*.png",
  { eager: true, query: "?url", import: "default" },
);
// icon@rarity; older saves have no rarity
const parts = computed(() =>
  props.chest.icons?.map((entry) => entry.split("@")),
);
const srcs = computed(() =>
  parts.value?.map(([icon]) => ART[`${DIR}${icon}.png`]),
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
      :style="rarity_bg(parts![i][1])"
      class="piece-icon generic-icon h-8 w-8"
      :alt="chest.title"
    />
  </span>
  <span v-else>{{ chest.title }}</span>
</template>
