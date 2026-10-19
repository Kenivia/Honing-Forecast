<script setup lang="ts">
import { useRosterStore as useRosterStore } from "@/Stores/RosterConfig";
import { CONVERTIBLE_MATERIALS } from "@/Utils/Constants";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import TierConvertButton from "../Common/TierConvertButton.vue";
import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher";
import { input_column_to_num, set_cell } from "@/Utils/InputColumn";
import Sidebar from "../Common/Sidebar.vue";
import BudgetGrid from "./BudgetGrid.vue";
import RegionSelector from "../Common/RegionSelector.vue";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();

const roster_store = useRosterStore();
const { roster_config, roster_ids } = storeToRefs(roster_store);

const selected_roster_id = ref(roster_ids.value[0]);

const selected_roster_mats_owned = computed(
  () => roster_config.value.roster_mats_owned[selected_roster_id.value],
);
const selected_tradable_mats_owned = computed(
  () => roster_config.value.tradable_mats_owned[selected_roster_id.value],
);
const selected_region = computed(
  () => roster_config.value.all_regions[selected_roster_id.value],
);
// Converting up a tier: each unit of the new material eats `ratio` of the source, and
// what comes out is roster-bound regardless of which pool it came from.
function convert_roster_mats_to_serca() {
  const roster = selected_roster_mats_owned.value;
  const tradable = selected_tradable_mats_owned.value;
  for (const mat of CONVERTIBLE_MATERIALS) {
    const source = mat.from.label;
    if (roster.values[source] === undefined) {
      continue;
    }
    const pooled =
      input_column_to_num(roster)[source] +
      input_column_to_num(tradable)[source];
    const gained = Math.floor(pooled / mat.from.ratio);
    set_cell(
      roster,
      mat.label,
      String(input_column_to_num(roster)[mat.label] + gained),
    );
    set_cell(roster, source, "0");
    set_cell(tradable, source, "0");
  }
}
</script>

<template>
  <Sidebar :width="1315" header="Market & mats">
    <template #sidebar>
      <div class="side-bar-item">
        <RegionSelector
          :region="selected_region"
          :region_change="
            (event) => {
              const new_region = (event.target as HTMLSelectElement)
                .value as MarketRegions;
              roster_config.all_regions[selected_roster_id] = new_region;
              start_fetch(new_region);
            }
          "
        />

        <div class="flex flex-row">
          <span v-if="selected_region !== 'Custom' && roster_config.auto_fetch">
            {{
              !runtime.is_fetching && !runtime.market_fetch_failed
                ? "✅"
                : runtime.is_fetching
                  ? ""
                  : "Failed"
            }}
          </span>
          <button
            :disabled="runtime.is_fetching || selected_region === 'Custom'"
            @click="() => start_fetch(selected_region, true)"
            class="generic-button mx-3! w-max!"
            :style="{
              opacity: selected_region === 'Custom' ? 0.5 : 1,
              cursor: selected_region === 'Custom' ? 'not-allowed' : 'pointer',
            }"
          >
            {{ !runtime.is_fetching ? "Fetch Market Data" : "Fetching..." }}
          </button>
        </div>
        <div
          class="control-panel-checkbox-row border-0!"
          v-if="selected_region !== 'Custom'"
        >
          <span>Auto fetch </span>
          <input
            type="checkbox"
            v-model="roster_config.auto_fetch"
            @change="() => start_fetch(selected_region)"
          />
        </div>
      </div>

      <div v-if="roster_ids.length > 1" class="side-bar-item">
        <span class="text-nowrap"> Active Roster: </span>
        <select v-model="selected_roster_id" class="selector">
          <option
            v-for="(roster_id, roster_index) in roster_ids"
            :value="roster_id"
            :key="roster_id"
          >
            Roster {{ roster_index + 1 }}
          </option>
        </select>
      </div>
      <TierConvertButton
        label-text="Convert owned T4 Roster & Tradable to T4.5 Serca mats (5 to 1 ratio)"
        tooltip-text="Red, Blue, and Leaps (not abidos)"
        @change-tier="convert_roster_mats_to_serca"
      />
    </template>

    <template #main>
      <BudgetGrid :selected_roster_id="selected_roster_id" />
    </template>
  </Sidebar>
</template>
<style>
.shard-size-selector {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;

  color: var(--text-muted);
  flex-direction: column;
}
</style>
