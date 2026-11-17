<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  export_config,
  import_config,
  write_state,
} from "@/Stores/ConfigStorage";
import { storeToRefs } from "pinia";
import { ref } from "vue";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { grid_change_callback } from "./CharWorkerUtils";

const store = useRosterStore();
const { active_profile, roster_config } = storeToRefs(store);

// the old results are of the old tier's labels, so they go first, as in change_tier
function resetActive() {
  const runtime = useRuntimeStore().for_char(active_profile.value.id);
  runtime?.optimizer.cancel_and_clear_prev_result();
  runtime?.histogram.cancel_and_clear_prev_result();
  runtime?.marginal.cancel_and_clear_prev_result();
  store.reset_active_profile();
  grid_change_callback();
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
    // reload so runtime state and workers rebuild from the imported config
    store.replace_config(import_config(await file.text()));
    write_state(store.$state);
    location.reload();
  } catch (e) {
    import_error.value = e instanceof Error ? e.message : String(e);
  }
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
      <label class="control-panel-checkbox-row">
        <input v-model="active_profile.lock_fetched_done" type="checkbox" />
        <span>Lock upgrades fetched from lostark.bible</span>
      </label>
      <label>
        <button
          class="generic-button ml-5 text-(--warning-dark)!"
          aria-label="Reset this char"
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
