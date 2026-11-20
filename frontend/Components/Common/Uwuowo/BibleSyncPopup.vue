<script setup lang="ts">
import { close_sync_popup, sync_pending } from "@/Utils/BibleSync";
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import Popup from "../Popup.vue";

const { roster_ids } = storeToRefs(useRosterStore());
</script>

<template>
  <Popup
    :show_popup="sync_pending.length > 0"
    label="Match your characters"
    @close_popup="close_sync_popup(false)"
  >
    <div class="flex flex-col gap-4">
      <span class="text-2xl">Match your characters with lostark.bible</span>
      <!-- <span class="text-(--text-muted)"
        >These names were not found on lostark.bible.</span
      > -->
      <div v-for="entry in sync_pending" :key="entry.roster_id">
        <span class="font-bold"
          >Roster {{ roster_ids.indexOf(entry.roster_id) + 1 }}</span
        >
        <div class="grid grid-cols-[max-content_1fr] items-center gap-2">
          <template v-for="local in entry.locals" :key="local.id">
            <span>{{ local.name }}</span>
            <select
              v-model="local.choice"
              class="generic-input"
              :aria-label="`Match ${local.name}`"
            >
              <option value="">Not on lostark.bible</option>
              <option
                v-for="name in entry.bibles"
                :key="name"
                :value="name"
                :disabled="
                  entry.locals.some((l) => l !== local && l.choice === name)
                "
              >
                {{ name }}
              </option>
            </select>
          </template>
        </div>
      </div>
      <div class="flex flex-row gap-2">
        <button
          class="generic-button w-full text-(--achieved)!"
          @click="close_sync_popup(true)"
        >
          Confirm
        </button>
        <button class="generic-button w-full" @click="close_sync_popup(false)">
          Skip
        </button>
      </div>
    </div>
  </Popup>
</template>
