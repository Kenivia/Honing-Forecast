<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import Instructions from "./Instructions/Instructions.vue";
import MaterialDist from "./MaterialDist/MaterialDist.vue";
import OptimizerControlPanel from "./OptimizerControlPanel/OptimizerControlPanel.vue";
import StatusInput from "./StatusInput/StatusInput.vue";
import ChestPanels from "@/Components/Common/ChestPanels.vue";
import ChestState from "./ChestState.vue";
import ChestEditor from "./ChestEditor.vue";
import { storeToRefs } from "pinia";
import { start_all_workers, start_eval_hist } from "./CharWorkerUtils.js";
import { useRosterStore } from "@/Stores/RosterConfig.js";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();
const {
  active_profile,
  active_region,
  active_chests,
  processed_chests: chests,
} = storeToRefs(useRosterStore());
// the stored chests are shown in place of these while editing
const editing_chests = ref(false);
// the only ones with a choice to show
const selection = computed(() =>
  chests.value.filter((chest) => chest.options.length > 1),
);

watch(
  [
    () => active_profile.value.express_event,
    () => active_profile.value.band_values,
    () => active_chests.value,
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
  runtime.cancel_active();
});
</script>
<template>
  <StatusInput />
  <MaterialDist />
  <label class="flex w-fit items-center gap-1 text-sm">
    <input v-model="editing_chests" type="checkbox" />
    Edit chests
  </label>
  <ChestEditor v-if="editing_chests" />
  <ChestPanels
    v-else
    :selection="selection"
    :obtain_all="chests.filter((chest) => chest.options.length === 1)"
    obtain_all_title="Obtain-all and trivial selection chests"
  />
  <ChestState :chests="selection" />
  <OptimizerControlPanel />
  <Instructions :is_normal="true" />
  <Instructions :is_normal="false" />
</template>
