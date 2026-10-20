<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  broken_backup,
  broken_save,
  discard_broken,
} from "@/Stores/ConfigStorage";

const store = useRosterStore();

function download() {
  const blob = new Blob([broken_backup(broken_save.value)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = "honing-forecast-unreadable-save.json";
  link.click();
  URL.revokeObjectURL(url);
}
</script>

<template>
  <div
    v-if="broken_save"
    role="alert"
    class="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-(--warning-dark) bg-(--bg-muted) px-4 py-2 text-sm"
  >
    <span class="text-(--warning)">
      Your saved data couldn't be loaded and was kept aside.
      <template v-if="!broken_save.kept">
        There is no room to keep a copy, so saving is paused until you download
        or discard it.
      </template>
      Download it to restore later with Backup → Import.
    </span>
    <button class="generic-button" @click="download">Download</button>
    <button class="generic-button" @click="discard_broken(store)">
      Discard
    </button>
  </div>
</template>
