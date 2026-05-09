<script setup lang="ts">
import Instructions from "@/Components/Character/Instructions/Instructions.vue";
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue";
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue";
import { useRosterStore } from "@/stores/roster_config";
import { storeToRefs } from "pinia";
import { onUnmounted, watch } from "vue";
import { RouterLink, useRoute, useRouter } from "vue-router";
import ControlPanel from "./ControlPanel.vue";
import Sidebar from "../Common/SideBar.vue";
import { start_all_workers } from "./char_worker_util";
import GraphControlPanel from "./GraphControlPanel.vue";
import Guide from "./guide.vue";
import DetailedInput from "./StatusInput/DetailedInput.vue";

const route = useRoute();
const router = useRouter();

const roster_store = useRosterStore();
const { active_profile, all_profiles } = storeToRefs(roster_store);

const match = all_profiles.value.findIndex((c) => c.char_name === (route.params.characterName as string));
if (match >= 0) {
    roster_store.switchProfile(match);
} else {
    router.replace({ name: "char", params: { characterName: all_profiles.value[0].char_name } });
    roster_store.switchProfile(0);
}
watch(
    () => route.params.characterName as string,
    (name) => {
        const match = all_profiles.value.findIndex((c) => c.char_name === name);
        if (match >= 0) {
            if (roster_store.roster_config.active_profile_index !== match) {
                // this happens one invalid names (routre param written to by the one-off code, triggering the watcher) i believe, idk how to prevent that but this works
                active_profile.value.optimizer_worker_bundle.cancel();
                active_profile.value.histogram_worker_bundle.cancel();
                // active_profile.value.evaluation_worker_bundle.cancel()

                roster_store.switchProfile(match);
            }
        } else {
            router.replace({ name: "char", params: { characterName: all_profiles.value[0].char_name } });
            roster_store.switchProfile(0);
        }
    },
);
watch(
    [
        () => active_profile.value.express_event,
        () => active_profile.value.optimizer_treatment_plan,
        () => active_profile.value.auto_start_optimizer,
    ],
    () => {
        // console.log("start", active_profile.value, roster_config.value)
        start_all_workers();
    },
    { deep: true, immediate: true },
);
onUnmounted(() => {
    // kill workers when going to market / roster view
    active_profile.value.optimizer_worker_bundle.cancel();
    active_profile.value.histogram_worker_bundle.cancel();
    // active_profile.value.evaluation_worker_bundle.cancel()
});
</script>
<template>
    <Sidebar :header="active_profile.char_name">
        <template #sidebar="{ close }">
            <div style="display: flex; flex-direction: column">
                <RouterLink to="guide" class="hf-side-bar-item" @click="close"> Guide </RouterLink>
                <RouterLink to="calc" class="hf-side-bar-item" @click="close"> Setup & Cost Analysis </RouterLink>
                <RouterLink to="instructions" class="hf-side-bar-item" @click="close"> Taps Instructions </RouterLink>
            </div>

            <GraphControlPanel v-if="route.path.endsWith('calc')" />
            <ControlPanel v-if="route.path.endsWith('calc')" />
        </template>

        <template #main>
            <StatusInput v-if="route.path.endsWith('calc')" />
            <MaterialDist v-if="route.path.endsWith('calc')" />
            <DetailedInput v-if="route.path.endsWith('calc')" />
            <Instructions v-if="route.path.endsWith('calc')" />
            <Guide v-if="route.path.endsWith('guide')" />
        </template>
    </Sidebar>
</template>
