<script setup lang="ts">
import Sidebar from "@/Components/Common/SideBar.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { ALL_VERSIONS, LATEST_VERSION } from "@/Utils/Changelog";
import { storeToRefs } from "pinia";
import { ref, watchEffect } from "vue";
import { RouterLink, useRoute } from "vue-router";
import { marked } from "marked";

const route = useRoute();

const { roster_config } = storeToRefs(useRosterStore());
roster_config.value.last_seen_version = LATEST_VERSION;

const html = ref("");

const cache = new Map<string, string>();
const loading = ref(false);

watchEffect(async () => {
  const version = route.params.version as string;

  if (cache.has(version)) {
    html.value = cache.get(version)!;
    return;
  }

  loading.value = true;
  const res = await fetch(
    version === "WIP" ? `/WIP.md` : `/change-logs/${version}.md`,
  );
  const raw = await res.text();
  const parsed = await marked.parse(raw);
  cache.set(version, parsed);
  html.value = parsed;
  loading.value = false;
});
</script>

<template>
  <Sidebar header="Change Logs">
    <template #sidebar="{ close }">
      <div class="flex flex-col">
        <RouterLink
          :to="{ name: 'change-logs', params: { version: 'WIP' } }"
          class="side-bar-link"
          @click="close"
        >
          Work in Progress
        </RouterLink>
        <RouterLink
          v-for="version in ALL_VERSIONS"
          :to="'/change-logs/' + version"
          class="side-bar-link"
          @click="close"
          :key="version"
        >
          {{ version }}
        </RouterLink>
      </div>
    </template>

    <template #main>
      <div class="card-shell change-log-card p-3">
        <div class="text-9xl; text-center; mt-20" v-if="loading">
          Loading...
        </div>
        <div v-else class="prose" v-html="html" /></div
    ></template>
  </Sidebar>
</template>

<!-- this style cannot be scoped cos it needs to go deep into the html -->
<style>
.change-log-card {
  width: min(100%, 1000px);
}
.prose h1 {
  font-size: 3rem;
  font-weight: 600;
  margin-bottom: 1rem;
}

.prose li {
  list-style-type: " - ";
  list-style-position: inside;
}
.prose h2 {
  margin-top: 1.5rem;
  margin-bottom: 0.5rem;
  font-size: x-large;
  font-weight: 500;
  border-bottom: 1px solid var(--border-medium);
}
.prose tr {
  display: grid;
  grid-template-columns: 100px calc(100% - 100px - 80px - 50px - 30px) 80px 50px;
  border-bottom: 1px solid var(--border-subtle);
  align-items: flex-start;
}
@media (max-width: 480px) {
  .prose tr {
    display: flex;
    flex-direction: column;
    border-bottom: 1px solid var(--border-subtle);
    align-items: flex-start;
    justify-content: space-between;
  }
}
.prose td {
  align-items: flex-start;
}
.prose table {
  display: flex;
  flex-direction: column;
  border-collapse: collapse;
}
.prose th {
  text-align: left;
  font-size: small;
  display: grid;
}

.prose a {
  color: var(--gold);
  text-decoration: underline;
}
</style>
