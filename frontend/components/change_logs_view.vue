<script setup lang="ts">
import Sidebar from "@/Components/Common/SideBar.vue";
import { useRosterStore } from "@/stores/roster_config";
import { ALL_VERSIONS, LATEST_VERSION } from "@/utils/change_log";
import { storeToRefs } from "pinia";
import { ref, watchEffect } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
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
    const res = await fetch(version === "WIP" ? `/WIP.md` : `/change-logs/${version}.md`);
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
            <div style="display: flex; flex-direction: column">
                <RouterLink
                    :to="{ name: 'change-logs', params: { version: 'WIP' } }"
                    class="hf-side-bar-item"
                    @click="close"
                >
                    Work in Progress
                </RouterLink>
                <RouterLink
                    v-for="version in ALL_VERSIONS"
                    :to="'/change-logs/' + version"
                    class="hf-side-bar-item"
                    @click="close"
                    :key="version"
                >
                    {{ version }}
                </RouterLink>
            </div>
        </template>

        <template #main>
            <div class="hf-card change-log-card">
                <div style="font-size: 60px; text-align: center; margin-top: 70px" v-if="loading">Loading...</div>
                <div v-else class="prose" v-html="html" /></div
        ></template>
    </Sidebar>
</template>
<style>
.change-log-card {
    width: min(100%, 1000px);
    padding: 12px;
}
.prose h2 {
    border-bottom: 1px solid var(--hf-border);
}
.prose tr {
    display: grid;
    grid-template-columns: 80px 710px 80px 50px;
    border-bottom: 1px solid var(--hf-border-subtle);
    gap: 10px;
    align-items: flex-start;
}
.prose td {
    align-items: flex-start;
}
.prose table {
    display: table;
    border-collapse: collapse;
}
.prose th {
    text-align: left;
}

.prose a {
    color: var(--hf-gold);
    text-decoration: underline;
}
</style>
