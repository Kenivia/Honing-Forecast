<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import Instructions from "./Instructions/Instructions.vue";
import MaterialDist from "./MaterialDist/MaterialDist.vue";
import OptimizerControlPanel from "./OptimizerControlPanel/OptimizerControlPanel.vue";
import StatusInput from "./StatusInput/StatusInput.vue";
import ChestPanels from "@/Components/Common/ChestPanels.vue";
import ChestEditor from "./ChestEditor.vue";
import { storeToRefs } from "pinia";
import { start_all_workers } from "./CharWorkerUtils.js";
import { useRosterStore } from "@/Stores/RosterConfig.js";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { settle_deductions } from "./Instructions/Details/NormalDetails/SuccessUtils";
import { process_chests } from "@/Utils/Chest";
const runtime = useRuntimeStore();
const store = useRosterStore();
const {
  active_profile,
  active_region,
  active_chests,
  active_mats_prices,
  active_selected_bundles,
  processed_chests,
  forbidden_processed_chests: forbidden,
} = storeToRefs(store);
// the forbidden ones are shown too, greyed
const chests = computed(() => [...processed_chests.value, ...forbidden.value]);
const forbidden_keys = computed(() =>
  forbidden.value.map((chest) => chest.key),
);
// a processed key stands for every stored chest that merges into it
const forbid = (key: string) => {
  if (forbidden_keys.value.includes(key)) return store.toggle_forbid(key);
  for (const chest of store.allowed_chests)
    if (process_chests([chest], active_profile.value.tier)[0]?.key === key)
      store.toggle_forbid(chest.key);
};
// the stored chests are shown in place of these while editing
const editing_chests = ref(false);
const chests_open = ref(true);
const show_obtain_all = ref(false);
// the wide chest card takes its own row, the thin one sits beside the optimizer
const chests_wide = computed(
  () => editing_chests.value || show_obtain_all.value,
);
// both views, editing or not, put the chests with a choice on the left
const obtain_all_title = "Obtain-all and trivial selection chests";
const selection = computed(() =>
  chests.value.filter((chest) => chest.options.length > 1),
);

watch(
  [
    () => active_profile.value.express_event,
    () => active_profile.value.band_values,
    () => active_chests.value,
    () => active_profile.value.forbidden_chests,
    () => active_profile.value.auto_start_optimizer,
    () => active_profile.value.min_resolution,
    () => active_region.value,
    // a fetch landing after mount; by value, as every fetch rewrites the bundles
    () =>
      JSON.stringify([
        active_mats_prices.value.values,
        active_selected_bundles.value,
      ]),
  ],
  // with auto start off this only restores a saved run, else draws the histogram
  () => start_all_workers(),
  { deep: true, immediate: true },
);
// A character switch remounts this page (CharView keys it), and this runs after the next
// character's page has started, so it only touches its own character.
const own = active_profile.value;
onUnmounted(() => {
  const bundles = runtime.for_char(own.id);
  bundles?.optimizer.cancel();
  bundles?.histogram.cancel();
  bundles?.marginal.cancel();
  settle_deductions(own);
});
</script>
<template>
  <StatusInput />
  <MaterialDist />
  <div
    class="flex max-w-full gap-2.5"
    :class="
      chests_wide
        ? 'flex-col items-center'
        : 'flex-wrap items-start justify-center'
    "
  >
    <ChestEditor
      v-if="editing_chests"
      v-model:open="chests_open"
      v-model:show_obtain_all="show_obtain_all"
      :obtain_all_title="obtain_all_title"
    >
      <template #controls>
        <label class="flex items-center gap-1">
          <input v-model="editing_chests" type="checkbox" />
          Edit chests
        </label>
      </template>
    </ChestEditor>
    <ChestPanels
      v-else
      v-model:open="chests_open"
      v-model:show_obtain_all="show_obtain_all"
      :selection="selection"
      :obtain_all="chests.filter((chest) => chest.options.length === 1)"
      selection_title="Chest opening advice"
      :obtain_all_title="obtain_all_title"
      :hide_count="true"
      :states="runtime.histogram.result?.state_bundle.chest_states"
      :forbidden="forbidden_keys"
      @forbid="forbid"
    >
      <template #controls>
        <label class="flex items-center gap-1">
          <input v-model="editing_chests" type="checkbox" />
          Edit chests
        </label>
      </template>
    </ChestPanels>
    <OptimizerControlPanel class="self-center" />
  </div>
  <Instructions :is_normal="true" />
  <Instructions :is_normal="false" />
</template>
