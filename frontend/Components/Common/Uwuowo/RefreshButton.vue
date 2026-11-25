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
    <Spinner v-if="refreshing.has(profile_id)" class="h-4 w-4" />
    <span
      v-else
      class="block h-4 w-4 bg-current"
      style="
        mask: url(/Icons/rotate-right-solid-full.svg) center / contain no-repeat;
      "
    />
  </button>
</template>
