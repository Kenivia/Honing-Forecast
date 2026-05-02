<script setup lang="ts">
import Sidebar from "@/Components/Common/Sidebar.vue"
import { useRosterStore } from "@/Stores/RosterConfig"
import { all_change_logs, ALL_VERSIONS, LATEST_VERSION } from "@/Utils/Changelog"
import { storeToRefs } from "pinia"
import { computed, defineAsyncComponent, onMounted, ref } from "vue"
import { RouterLink, RouterView, useRoute } from "vue-router"
import { marked } from "marked"

const route = useRoute()

const { roster_config } = storeToRefs(useRosterStore())
roster_config.value.last_seen_version = LATEST_VERSION

const html = ref("")

onMounted(async () => {
    const res = await fetch(`/change-logs/${route.params.version}.md`)
    const raw = await res.text()

    html.value = await marked.parse(raw)
    console.log(html.value)
})
</script>

<template>
    <Sidebar header="Change Logs">
        <template #sidebar="{ close }">
            <RouterLink v-for="version in ALL_VERSIONS" :to="'/change-logs/' + version" class="hf-side-bar-item" @click="close"> {{ version }} </RouterLink>
        </template>

        <template #main key="changelog">
            <div class="hf-card change-log-card">
                <div class="prose" v-html="html" /></div
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
    color: var(--hf-text-muted);
    text-decoration: underline;
}
</style>
