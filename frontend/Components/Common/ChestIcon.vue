<script setup lang="ts">
import type { Chest } from "@/Utils/Chest";

// A stack's art, its title on hover; a chest without art shows its title.
defineProps<{ chest: Chest }>();

const DIR = "../../../scanner_setup/templates/ChestIcons/";
const ART = import.meta.glob<string>(
  "../../../scanner_setup/templates/ChestIcons/*.png",
  { eager: true, query: "?url", import: "default" },
);
</script>

<template>
  <span
    v-if="chest.icons?.length"
    class="flex items-center gap-0.5"
    :title="chest.title"
  >
    <img
      v-for="icon in chest.icons"
      :key="icon"
      :src="ART[`${DIR}${icon}.png`]"
      class="generic-icon h-8 w-8"
      :alt="chest.title"
    />
  </span>
  <span v-else>{{ chest.title }}</span>
</template>
