<script setup lang="ts">
import Sidebar from "@/Components/Common/Sidebar.vue";
import { ALL_VERSIONS } from "@/Utils/Changelog";
import { RouterLink, RouterView } from "vue-router";

const DOCS = [
  { name: "info-guide", label: "Guide" },
  { name: "info-upgrade-order", label: "What are we doing here?" },
];
</script>

<template>
  <Sidebar :width="1200" header="Info">
    <template #sidebar="{ close }">
      <div class="flex flex-col">
        <span class="px-2 pt-2 pb-1 text-sm text-(--text-muted)">Docs</span>
        <RouterLink
          v-for="doc in DOCS"
          :key="doc.name"
          :to="{ name: doc.name }"
          class="side-bar-link"
          @click="close"
        >
          {{ doc.label }}
        </RouterLink>
        <span class="px-2 pt-4 pb-1 text-sm text-(--text-muted)"
          >Change logs</span
        >
        <RouterLink
          :to="{ name: 'change-logs', params: { version: 'WIP' } }"
          class="side-bar-link"
          @click="close"
        >
          Work in Progress
        </RouterLink>
        <RouterLink
          v-for="version in ALL_VERSIONS"
          :key="version"
          :to="{ name: 'change-logs', params: { version } }"
          class="side-bar-link"
          @click="close"
        >
          {{ version }}
        </RouterLink>
      </div>
    </template>

    <template #main>
      <RouterView />
    </template>
  </Sidebar>
</template>
