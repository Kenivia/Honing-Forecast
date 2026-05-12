<script setup lang="ts">
import { TreatmentPlan } from "@/Stores/CharacterProfile";
import { useRosterStore } from "@/Stores/RosterConfig";
// import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { storeToRefs } from "pinia";
import { ref, watchEffect } from "vue";

const store = useRosterStore();
const { active_profile, roster_config } = storeToRefs(store);

function resetActive() {
  store.resetActiveProfile();
}

// function copyPayload() {
//   const payload = JSON.stringify(build_payload(), null, 2);
//   navigator.clipboard?.writeText(payload).catch(() => undefined);
// }

// Currently TreatRosterAsTradable is not selectable
const treatment_tick = ref(
  active_profile.value.optimizer_treatment_plan ==
    TreatmentPlan.TreatRosterAsBound,
);
watchEffect(() => {
  // console.log("changed")
  if (treatment_tick.value) {
    active_profile.value.optimizer_treatment_plan =
      TreatmentPlan.TreatRosterAsBound;
  } else {
    active_profile.value.optimizer_treatment_plan =
      TreatmentPlan.TreatTradableAsBound;
  }
});
</script>
<template>
  <section class="w-full items-center">
    <div class="control-panel-title">Controls</div>

    <div class="py-1 text-sm">
      <label class="control-panel-checkbox-row">
        <input v-model="active_profile.express_event" type="checkbox" />
        <span>Express event (March)</span>
      </label>
      <!-- This is for producing payloads to feed into Rust -->
      <!-- <button
        class="generic-button ml-5"
        @click="copyPayload"
      >
        Copy Payload
      </button> -->
      <label class="control-panel-checkbox-row">
        <input v-model="roster_config.show_all_rows" type="checkbox" />
        <span>Show all mats</span>
      </label>
      <label class="control-panel-checkbox-row">
        <input v-model="treatment_tick" type="checkbox" />
        <span>Account for sell value of tradable mats (Recommended)</span>
      </label>
      <label class="control-panel-checkbox-row">
        <input v-model="active_profile.auto_start_optimizer" type="checkbox" />
        <span>Auto start optimizer</span>
      </label>
      <label class="">
        <button
          class="generic-button ml-5 text-(--warning-dark)!"
          @click="resetActive"
        >
          Reset this char
        </button>
      </label>
    </div>
  </section>
</template>
