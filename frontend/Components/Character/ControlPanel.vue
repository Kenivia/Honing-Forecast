<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { export_config, import_config } from "@/Stores/ConfigStorage";
import {
  BAND_COLORS,
  BAND_LABELS,
  BAND_VALUES,
  BAND_VALUE_LABELS,
  BandValue,
  band_value_allowed,
} from "@/Utils/Constants";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import { storeToRefs } from "pinia";
import { ref } from "vue";

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

// What a material the character owns but does not end up spending is worth. Calc.vue
// watches band_values and restarts the workers.
const leftover_tooltip = `How much a material you don't end up spending is worth.
`;

// shown tradable first: it is the band people actually change, and the one below it can
// never be worth more, so the list reads downwards as "and nothing cheaper than this"
const BAND_ROWS = [...BAND_LABELS.keys()].reverse();

function set_band(band: number, event: Event) {
  const value = (event.target as HTMLSelectElement).value as BandValue;
  active_profile.value.band_values[band] = value;
}
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

      <div
        class="control-panel-title mt-2 flex flex-row items-center justify-between pr-2"
      >
        <span>Leftover mats are worth</span>
        <QuestionMark :text="leftover_tooltip" />
      </div>
      <div class="px-2 pt-1">
        <label
          v-for="band in BAND_ROWS"
          :key="band"
          class="flex flex-row items-center gap-2 py-0.5"
        >
          <span
            class="w-16 shrink-0 text-right"
            :style="{ color: `var(${BAND_COLORS[band]})` }"
            >{{ BAND_LABELS[band] }}</span
          >
          <select
            class="selector min-w-0 flex-1"
            :aria-label="`${BAND_LABELS[band]} leftover value`"
            :value="active_profile.band_values[band]"
            @change="set_band(band, $event)"
          >
            <option
              v-for="value in BAND_VALUES"
              :key="value"
              :value="value"
              :disabled="
                !band_value_allowed(active_profile.band_values, band, value)
              "
            >
              {{ BAND_VALUE_LABELS[value] }}
            </option>
          </select>
        </label>
      </div>

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
