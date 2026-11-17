<script setup lang="ts">
// The class emblem in the text colour (scripts/class_icons.mjs); the game's helmet for an unknown class.
import { computed } from "vue";
import { CLASS_KEYS as CLASSES } from "@/Utils/Constants";
import { class_key } from "@/Utils/Helpers";

const props = defineProps<{ class_name: string }>();
const key = computed(() => class_key(props.class_name));
const mask = (file: string) => ({
  mask: `url(/Icons/Classes/${file}) center / contain no-repeat`,
});
</script>

<template>
  <span
    v-if="CLASSES.has(key)"
    class="inline-block size-5 shrink-0 bg-current"
    :style="mask(`${key}.svg`)"
    :title="class_name"
    aria-hidden="true"
  />
  <span
    v-else
    class="relative grid size-5 shrink-0 place-items-center"
    title="Class not imported"
    aria-hidden="true"
  >
    <span
      class="absolute inset-0 bg-current opacity-50"
      :style="mask('unknown.png')"
    />
    <span class="relative text-sm leading-none font-bold after:content-['?']" />
  </span>
</template>
