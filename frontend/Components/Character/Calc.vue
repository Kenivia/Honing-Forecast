<script setup lang="ts">
import { computed, onUnmounted, watch } from "vue";
import Instructions from "./Instructions/Instructions.vue";
import MaterialDist from "./MaterialDist/MaterialDist.vue";
import OptimizerControlPanel from "./OptimizerControlPanel/OptimizerControlPanel.vue";
import StatusInput from "./StatusInput/StatusInput.vue";
import ChestPanels from "@/Components/Common/ChestPanels.vue";
import ChestState from "./ChestState.vue";
import { storeToRefs } from "pinia";
import { start_all_workers, start_eval_hist } from "./CharWorkerUtils.js";
import { useRosterStore } from "@/Stores/RosterConfig.js";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { process_chests } from "@/Utils/Chest";
const runtime = useRuntimeStore();
const { active_profile, active_region } = storeToRefs(useRosterStore());

// what the payload sends: the scanned chests as this character's tier takes them
const chests = computed(() =>
  process_chests(active_profile.value.chests, active_profile.value.tier),
);
// the only ones with a choice to show
const selection = computed(() =>
  chests.value.filter((chest) => chest.options.length > 1),
);

watch(
  [
    () => active_profile.value.express_event,
    () => active_profile.value.band_values,
    () => active_profile.value.chests,
    () => active_profile.value.auto_start_optimizer,
    () => active_region.value,
    () => active_profile.value.chests,
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
  runtime.cancel_active();
});
</script>
<template>
  <StatusInput />
  <MaterialDist />
  <ChestPanels
    :selection="selection"
    :obtain_all="chests.filter((chest) => chest.options.length === 1)"
    obtain_all_title="Obtain-all and trivial selection chests"
  />
  <ChestState :chests="selection" />
  <OptimizerControlPanel />
  <Instructions :is_normal="true" />
  <Instructions :is_normal="false" />
</template>
