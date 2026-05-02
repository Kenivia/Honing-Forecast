<script setup lang="ts">
import Sidebar from "@/Components/Common/Sidebar.vue"
import { useRosterStore } from "@/Stores/RosterConfig"
import { all_change_logs, ALL_VERSIONS, LATEST_VERSION } from "@/Utils/Changelog"
import { storeToRefs } from "pinia"
import { computed, defineAsyncComponent } from "vue"
import { RouterLink, RouterView, useRoute } from "vue-router"

const route = useRoute()

const selected_version_component = computed(() => {
    const loader = all_change_logs["/frontend/Changelogs/" + route.params.version + ".vue"]
    if (!loader) {
        return null
    }
    console.log("/frontend/Changelogs/" + route.params.version + ".vue", route.fullPath)
    return defineAsyncComponent(loader as () => Promise<unknown>)
})
const { roster_config } = storeToRefs(useRosterStore())
roster_config.value.last_seen_version = LATEST_VERSION
</script>

<template>
    <Sidebar header="Change Logs">
        <template #sidebar="{ close }">
            <RouterLink v-for="version in ALL_VERSIONS" :to="'/change-logs/' + version" class="hf-side-bar-item" @click="close"> {{ version }} </RouterLink>
        </template>

        <template #main key="changelog"> <component :is="selected_version_component" /></template>
    </Sidebar>
</template>
