<script setup lang="ts">
import Sidebar from "@/Components/Common/Sidebar.vue";
import Spinner from "@/Components/Common/Spinner.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  ALL_VERSIONS,
  LATEST_VERSION,
  load_change_log,
} from "@/Utils/Changelog";
import { computedAsync } from "@vueuse/core";
import { ref } from "vue";
import { RouterLink, useRoute } from "vue-router";

const route = useRoute();

useRosterStore().roster_config.last_seen_version = LATEST_VERSION;

// a stale fetch landing late is dropped by computedAsync; no version while leaving the page
const loading = ref(false);
const html = computedAsync(
  () => {
    const version = route.params.version as string | undefined;
    return version ? load_change_log(version) : "";
  },
  "",
  loading,
);
</script>

<template>
  <Sidebar :width="1200" header="Change Logs">
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
      <div class="card-shell prose-card">
        <Spinner
          v-if="loading"
          class="mx-auto my-20 h-24 w-24 text-(--text-muted)"
        />
        <div v-else-if="html === null" class="text-(--warning)">
          Could not load this change log.
        </div>
        <div v-else class="change-log-html" v-html="html" />
      </div>
    </template>
  </Sidebar>
</template>

<!-- v-html content, so :deep -->
<style scoped>
.change-log-html :deep(tr) {
  display: grid;
  grid-template-columns: 100px calc(100% - 100px - 80px - 50px - 30px) 80px 50px;
  border-bottom: 1px solid var(--border-muted);
  align-items: flex-start;
}
@media (max-width: 480px) {
  .change-log-html :deep(tr) {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
}
.change-log-html :deep(td) {
  align-items: flex-start;
}
.change-log-html :deep(table) {
  display: flex;
  flex-direction: column;
  border-collapse: collapse;
}
.change-log-html :deep(th) {
  text-align: left;
  font-size: small;
  display: grid;
}
.change-log-html :deep(ul),
.change-log-html :deep(ol) {
  padding-left: 1.25rem;
  margin-bottom: 0.5rem;
}
.change-log-html :deep(li) {
  list-style-type: " - ";
  list-style-position: outside;
  margin-bottom: 0.25rem;
}
.change-log-html :deep(li > ul),
.change-log-html :deep(li > ol) {
  margin-top: 0.25rem;
  margin-bottom: 0;
}
</style>
