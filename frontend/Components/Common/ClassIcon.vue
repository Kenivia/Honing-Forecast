<script setup lang="ts">
// The class emblem in the text colour (scripts/class_icons.mjs); the game's helmet for an unknown class.
import { computed } from "vue";

const CLASSES = new Set([
  "aeromancer",
  "arcanist",
  "artillerist",
  "artist",
  "bard",
  "berserker",
  "breaker",
  "deadeye",
  "deathblade",
  "destroyer",
  "dimensionalist",
  "glaivier",
  "guardianknight",
  "gunlancer",
  "gunslinger",
  "machinist",
  "paladin",
  "reaper",
  "scrapper",
  "shadowhunter",
  "sharpshooter",
  "slayer",
  "sorceress",
  "souleater",
  "soulfist",
  "striker",
  "summoner",
  "valkyrie",
  "wardancer",
  "wildsoul",
]);

const props = defineProps<{ class_name: string }>();
const key = computed(() =>
  props.class_name.toLowerCase().replace(/[^a-z]/g, ""),
);
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
