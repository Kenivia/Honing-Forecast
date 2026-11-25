<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { grid_change_callback } from "./CharWorkerUtils";
import { computed, ref } from "vue";
import Popup from "../Common/Popup.vue";
import HoverTip from "../Common/HoverTip.vue";
import { linked_now } from "@/Utils/BibleAuth";
import { market_to_uwuowo_region } from "../Common/Uwuowo/UwuowoUtils";

const store = useRosterStore();
const { active_profile, roster_config } = storeToRefs(store);
const confirming = ref(false);

const lock_tip = computed(() => {
  const p = active_profile.value;
  const region = market_to_uwuowo_region(
    roster_config.value.all_regions[p.roster_id],
  );
  return region !== null && linked_now(region, p.char_name) !== null
    ? "Unlocking also pauses this character's automatic refresh from lostark.bible"
    : null;
});

// the old results are of the old tier's labels, so they go first, as in change_tier
function resetActive() {
  confirming.value = false;
  const runtime = useRuntimeStore().for_char(active_profile.value.id);
  runtime?.optimizer.cancel_and_clear_prev_result();
  runtime?.histogram.cancel_and_clear_prev_result();
  runtime?.marginal.cancel_and_clear_prev_result();
  store.reset_active_profile();
  grid_change_callback();
}
</script>
<template>
  <div class="w-full items-center">
    <div class="control-panel-title">Controls</div>

    <div class="py-1 text-sm">
      <label class="control-panel-checkbox-row">
        <input v-model="roster_config.show_all_rows" type="checkbox" />
        <span>Show all mats </span>
      </label>
      <label class="control-panel-checkbox-row">
        <input v-model="roster_config.auto_deduct_costs" type="checkbox" />
        <span>Slider auto-deducts costs</span>
      </label>
      <HoverTip :text="lock_tip" placement="right">
        <label class="control-panel-checkbox-row">
          <input v-model="active_profile.lock_fetched_done" type="checkbox" />
          <span>Lock upgrades fetched from lostark.bible</span>
        </label>
      </HoverTip>

      <label class="control-panel-checkbox-row">
        <input v-model="roster_config.show_suggestions" type="checkbox" />
        <span>Show suggestions</span>
      </label>
      <label>
        <button
          class="generic-button ml-5 text-(--warning-dark)!"
          aria-label="Reset this char"
          @click="confirming = true"
        >
          Reset this char
        </button>
      </label>
    </div>
    <Popup :show_popup="confirming" @close_popup="confirming = false">
      <div class="flex flex-col gap-4">
        <span class="text-2xl"
          >Reset {{ store.char_label(active_profile) }}?</span
        >
        <span class="text-(--text-main)"
          >Delete all upgrades and materials of this character
        </span>

        <div class="flex flex-row gap-2">
          <button
            class="generic-button w-fit text-(--warning-dark)!"
            @click="resetActive"
          >
            Reset
          </button>
          <button class="generic-button w-full" @click="confirming = false">
            Cancel
          </button>
        </div>
      </div>
    </Popup>
  </div>
</template>
