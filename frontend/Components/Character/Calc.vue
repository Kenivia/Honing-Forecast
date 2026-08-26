<script setup lang="ts">
import { onUnmounted, watch } from "vue";
import Instructions from "./Instructions/Instructions.vue";
import MaterialDist from "./MaterialDist/MaterialDist.vue";
import OptimizerControlPanel from "./OptimizerControlPanel/OptimizerControlPanel.vue";
import StatusInput from "./StatusInput/StatusInput.vue";
import { storeToRefs } from "pinia";
import { start_all_workers, start_eval_hist } from "./CharWorkerUtils.js";
import { useRosterStore } from "@/Stores/RosterConfig.js";
const { active_profile, active_region } = storeToRefs(useRosterStore());

watch(
  [
    () => active_profile.value.express_event,
    () => active_profile.value.optimizer_treatment_plan,
    () => active_profile.value.auto_start_optimizer,
    () => active_region.value,
  ],
  () => {
    // console.log("start", active_profile.value, roster_config.value)
    if (active_profile.value.auto_start_optimizer) {
      start_all_workers();
    } else {
      start_eval_hist();
    }
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
  <StatusInput />
  <MaterialDist />
  <OptimizerControlPanel />
  <Instructions :is_normal="true" />
  <Instructions :is_normal="false" />
</template>
