<script setup lang="ts">
import { sign_in, signed_in } from "@/Utils/BibleAuth";
import { useRosterStore } from "@/Stores/RosterConfig";
const roster_store = useRosterStore();
</script>

<template>
  <button
    v-if="!signed_in && roster_store.roster_config.show_suggestions"
    @click="sign_in"
    class="block cursor-pointer text-left text-(--uwuowo) underline"
  >
    Sign in with lostark.bible to fetch automatically
  </button>
</template>
