<script setup lang="ts">
import { handle_callback } from "@/Utils/BibleAuth";
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";

const router = useRouter();
const error = ref("");

onMounted(async () => {
  try {
    router.replace(await handle_callback());
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <div class="flex flex-col items-center gap-2 p-8">
    <span v-if="!error">Signing in with lostark.bible...</span>
    <template v-else>
      <span>Signing in with lostark.bible failed: {{ error }}</span>
      <router-link to="/" class="link">Back to Honing Forecast</router-link>
    </template>
  </div>
</template>
