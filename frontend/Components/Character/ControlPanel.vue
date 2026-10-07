<script setup lang="ts">
import { TreatmentPlan } from "@/Stores/CharacterProfile";
import { useRosterStore } from "@/Stores/RosterConfig";
import { export_config, import_config } from "@/Stores/ConfigStorage";
import { storeToRefs } from "pinia";
import { ref, watchEffect } from "vue";

const store = useRosterStore();
const { active_profile, roster_config } = storeToRefs(store);

function resetActive() {
  store.reset_active_profile();
}

const file_input = ref<HTMLInputElement | null>(null);
const import_error = ref("");

function download_backup() {
  const blob = new Blob([export_config(roster_config.value)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  const today = new Date().toISOString().slice(0, 10);
  link.download = `honing-forecast-${today}.json`;
  link.click();
  URL.revokeObjectURL(url);
}

async function load_backup(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = ""; // so picking the same file twice still fires
  if (!file) {
    return;
  }
  import_error.value = "";
  try {
    store.replace_config(import_config(await file.text()));
  } catch (e) {
    import_error.value = e instanceof Error ? e.message : String(e);
  }
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
  // console.log("changed");
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
  <div class="w-full items-center">
    <div class="control-panel-title">Controls</div>

    <div class="py-1 text-sm">
      <!-- <label class="control-panel-checkbox-row">
        <input v-model="active_profile.express_event" type="checkbox" />
        <span>Express event (June)</span>
      </label> -->
      <!-- This is for producing payloads to feed into Rust -->
      <!-- <button
        class="generic-button ml-5"
        @click="copyPayload"
      >
        Copy Payload
      </button> -->
      <label class="control-panel-checkbox-row">
        <input v-model="roster_config.show_all_rows" type="checkbox" />
        <span>Show all mats (this tier)</span>
      </label>
      <label class="control-panel-checkbox-row">
        <input v-model="treatment_tick" type="checkbox" />
        <span>Account for sell value of tradable mats (Recommended)</span>
      </label>

      <label class="control-panel-checkbox-row">
        <input v-model="roster_config.auto_deduct_costs" type="checkbox" />
        <span>Slider auto-deducts costs</span>
      </label>
      <label class="control-panel-checkbox-row">
        <input v-model="active_profile.lock_fetched_done" type="checkbox" />
        <span>Lock upgrades fetched from lostark.bible</span>
      </label>
      <label>
        <button
          class="generic-button ml-5 text-(--warning-dark)!"
          @click="resetActive"
        >
          Reset this char
        </button>
      </label>

      <div class="control-panel-title mt-2">Backup</div>
      <div class="flex flex-row flex-wrap items-center gap-2 px-2 pt-1">
        <button class="generic-button" @click="download_backup">
          Export to file
        </button>
        <button class="generic-button" @click="file_input?.click()">
          Import from file
        </button>
        <input
          ref="file_input"
          type="file"
          accept=".json,application/json"
          class="hidden"
          aria-label="Import backup file"
          @change="load_backup"
        />
        <span class="annotation basis-full"
          >Covers every character and roster, not just this one.</span
        >
        <span v-if="import_error" class="basis-full text-xs text-(--warning)">
          Import failed: {{ import_error }}
        </span>
      </div>
    </div>
  </div>
</template>
