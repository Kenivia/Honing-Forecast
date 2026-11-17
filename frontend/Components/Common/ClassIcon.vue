<script setup lang="ts">
// The class emblem in the text colour (scripts/class_icons.mjs); nothing for an unknown class.
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
</script>

<template>
  <span
    v-if="CLASSES.has(key)"
    class="inline-block size-5 shrink-0 bg-current"
    :style="{
      mask: `url(/Icons/Classes/${key}.svg) center / contain no-repeat`,
    }"
    :title="class_name"
    aria-hidden="true"
  />
</template>
