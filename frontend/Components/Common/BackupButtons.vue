<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  export_config,
  import_config,
  write_state,
} from "@/Stores/ConfigStorage";
import { storeToRefs } from "pinia";
import { ref } from "vue";

const store = useRosterStore();
const { roster_config } = storeToRefs(store);

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
  <div class="w-full items-center py-1 text-sm">
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
        >Covers every character and roster.</span
      >
      <span v-if="import_error" class="basis-full text-xs text-(--warning)">
        Import failed: {{ import_error }}
      </span>
    </div>
  </div>
</template>
