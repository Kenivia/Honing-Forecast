<script setup lang="ts">
import Instructions from "@/Components/Character/Instructions/Instructions.vue";
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue";
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue";
import { useRosterStore } from "@/Stores/RosterConfig";

import { storeToRefs } from "pinia";

import { onUnmounted, watch } from "vue";
import { RouterLink, useRoute, useRouter } from "vue-router";
import ControlPanel from "./ControlPanel.vue";

import Sidebar from "../Common/Sidebar.vue";
import { start_all_workers } from "./CharWorkerUtils";
import GraphControlPanel from "./GraphControlPanel.vue";
import Guide from "./Guide.vue";
import OptimizerControlPanel from "./OptimizerControlPanel.vue";
import NormalInstructions from "./Instructions/NormalInstructions.vue";
import AdvancedInstructions from "./Instructions/AdvancedInstructions.vue";

const route = useRoute();
const router = useRouter();

const roster_store = useRosterStore();
const { active_profile, all_profiles } = storeToRefs(roster_store);

const match = all_profiles.value.findIndex(
  (c) => c.char_name === (route.params.characterName as string),
);
if (match >= 0) {
  roster_store.switchProfile(match);
} else {
  router.replace({
    name: "char",
    params: { characterName: all_profiles.value[0].char_name },
  });
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
      router.replace({
        name: "char",
        params: { characterName: all_profiles.value[0].char_name },
      });
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
  <Sidebar
    :width="route.path.endsWith('calc') ? 1255 : 1200"
    :header="active_profile.char_name"
  >
    <template #sidebar="{ close }">
      <div class="flex flex-col">
        <RouterLink to="guide" class="side-bar-link" @click="close">
          Guide
        </RouterLink>
        <RouterLink to="calc" class="side-bar-link" @click="close">
          Setup & Cost Analysis
        </RouterLink>
      </div>

      <GraphControlPanel v-if="route.path.endsWith('calc')" />
      <ControlPanel v-if="route.path.endsWith('calc')" />
    </template>

    <template #main>
      <Guide v-if="route.path.endsWith('guide')" />
      <StatusInput v-if="route.path.endsWith('calc')" />
      <MaterialDist v-if="route.path.endsWith('calc')" />
      <OptimizerControlPanel v-if="route.path.endsWith('calc')" />
      <NormalInstructions v-if="route.path.endsWith('calc')" />
      <AdvancedInstructions v-if="route.path.endsWith('calc')" />
      <div class="min-h-30"></div>
    </template>
  </Sidebar>
</template>
