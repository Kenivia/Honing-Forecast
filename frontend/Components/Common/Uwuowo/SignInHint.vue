<script setup lang="ts">
import { sign_in, signed_in } from "@/Utils/BibleAuth";
</script>

<template>
  <button
    v-if="!signed_in"
    @click="sign_in"
    class="block cursor-pointer text-left text-xs text-(--text-very-muted) hover:underline"
  >
    Sign in with lostark.bible to fetch your own characters through their API
  </button>
</template>
