<script setup lang="ts">
import Spinner from "../Spinner.vue";
import IconButton from "../IconButton.vue";
import { refresh_char, refreshing } from "@/Utils/BibleSync";

defineProps<{ profile_id: string }>();
</script>

<template>
  <IconButton
    icon="rotate-right-solid-full.svg"
    aria-label="Refresh from lostark.bible"
    title="Refresh from lostark.bible"
    :disabled="refreshing.has(profile_id)"
    @click="refresh_char(profile_id)"
  >
    <Spinner v-if="refreshing.has(profile_id)" class="m-px h-full w-full" />
  </IconButton>
</template>
