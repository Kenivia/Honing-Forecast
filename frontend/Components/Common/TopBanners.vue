<script setup lang="ts">
// The strips at the top of every page's main area (Sidebar.vue); its own component so their updates
// don't re-render the page around them.
import Banner from "./Banner.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  broken_backup,
  broken_save,
  discard_broken,
  discard_moved_save,
  dismiss_moved_notice,
  moved_backup,
  moved_notice,
  moved_save,
  use_moved_save,
} from "@/Stores/ConfigStorage";
import { other_tabs } from "@/Utils/OtherTabs";
import { download_json } from "@/Utils/Helpers";
import { send_move_report } from "@/Utils/MoveReport";

const store = useRosterStore();
const OLD_SITE = import.meta.env.VITE_MIGRATE_FROM;
// const host = location.host;

// the store reloads too, or pagehide would write this site's save back
async function use_old_save() {
  use_moved_save(moved_save.value);
  store.init();
  await send_move_report({ outcome: "chose_old" });
  location.reload();
}

function download_old_save() {
  download_json(
    "honing-forecast-old-site-save.json",
    moved_backup(moved_save.value),
  );
  send_move_report({ outcome: "chose_download" });
}

function discard_old_save() {
  discard_moved_save();
  send_move_report({ outcome: "chose_discard" });
}
</script>

<template>
  <Banner v-if="moved_save">
    Your save from {{ OLD_SITE }} was not loaded, because this site already has
    one. Download it to keep as a backup, or use it in place of this one.
    <template #actions>
      <button class="generic-button" @click="download_old_save">
        Download
      </button>
      <button class="generic-button" @click="use_old_save">
        Use it instead
      </button>
      <button class="generic-button" @click="discard_old_save">Discard</button>
    </template>
  </Banner>
  <Banner v-else-if="moved_notice === 'failed'">
    Your saved data couldn't be moved from the old site.
    <a class="underline" :href="`${OLD_SITE}/roster-setup?stay`"
      >Use this link</a
    >, download a backup under Backup on the roster page, then import it here.
    <template #actions>
      <button class="generic-button" @click="dismiss_moved_notice">
        Dismiss
      </button>
    </template>
  </Banner>
  <Banner v-else-if="moved_notice === 'moved'" tone="info">
    Honing Forecast has moved to a real domain name!.
    <template #actions>
      <button class="generic-button" @click="dismiss_moved_notice">
        Dismiss
      </button>
    </template>
  </Banner>
  <Banner v-if="broken_save">
    Your saved data couldn't be loaded and was kept aside.
    <template v-if="!broken_save.kept">
      There is no room to keep a copy, so saving is paused until you download or
      discard it.
    </template>
    Download it to restore later with "Import backup from file" on the roster
    page.
    <template #actions>
      <button
        class="generic-button"
        @click="
          download_json(
            'honing-forecast-unreadable-save.json',
            broken_backup(broken_save),
          )
        "
      >
        Download
      </button>
      <button class="generic-button" @click="discard_broken(store)">
        Discard
      </button>
    </template>
  </Banner>
  <Banner v-if="other_tabs.length">
    Honing Forecast is open in another tab. Each tab overwrites the other's
    changes, so use one tab at a time.
  </Banner>
</template>
