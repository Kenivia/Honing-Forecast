<script setup lang="ts">
// The class emblem in the text colour (scripts/class_icons.mjs); the game's helmet for an unknown class.
// large: the parent sizes it and the "?" scales with it
import { computed } from "vue";
import { CLASS_KEYS as CLASSES } from "@/Utils/Constants";
import { class_key, icon_mask } from "@/Utils/Helpers";

const props = defineProps<{ class_name: string; large?: boolean }>();
const key = computed(() => class_key(props.class_name));
</script>

<template>
  <span
    v-if="CLASSES.has(key)"
    class="inline-block shrink-0 bg-current"
    :class="{ 'size-5': !large }"
    :style="icon_mask(`Classes/${key}.svg`)"
    :title="class_name"
    aria-hidden="true"
  />
  <span
    v-else
    class="relative grid shrink-0 place-items-center"
    :class="large ? '@container-size' : 'size-5'"
    :title="large ? undefined : 'Class not imported'"
    aria-hidden="true"
  >
    <span
      class="absolute inset-0 bg-current opacity-50"
      :style="icon_mask('Classes/unknown.png')"
    />
    <span
      class="relative leading-none font-bold after:content-['?']"
      :class="large ? 'text-[55cqh]' : 'text-sm'"
    />
  </span>
</template>
