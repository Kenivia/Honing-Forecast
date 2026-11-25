<script setup lang="ts">
import Spinner from "../Spinner.vue";
import { refresh_char, refreshing } from "@/Utils/BibleSync";

defineProps<{ profile_id: string }>();
</script>

<template>
  <button
    class="icon-button"
    aria-label="Refresh from lostark.bible"
    title="Refresh from lostark.bible"
    :disabled="refreshing.has(profile_id)"
    @click="refresh_char(profile_id)"
  >
    <Spinner v-if="refreshing.has(profile_id)" class="m-px h-full w-full" />
    <span
      v-else
      class="block h-full w-full bg-current"
      style="
        mask: url(/Icons/rotate-right-solid-full.svg) center / contain no-repeat;
      "
    />
  </button>
</template>
