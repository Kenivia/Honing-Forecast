<script setup lang="ts">
import { useRosterStore as useRosterStore } from "@/Stores/RosterConfig";
import { convert_pooled, CONVERT_SUMMARY } from "@/Utils/Convert";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import TierConvertButton from "../Common/TierConvertButton.vue";
import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher";
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
              runtime.is_fetching[selected_region]
                ? ""
                : runtime.market_fetch_failed[selected_region]
                  ? "Failed"
                  : "✅"
            }}
          </span>
          <button
            :disabled="
              runtime.is_fetching[selected_region] ||
              selected_region === 'Custom'
            "
            @click="() => start_fetch(selected_region, true)"
            class="generic-button mx-3! w-max!"
            :style="{
              opacity: selected_region === 'Custom' ? 0.5 : 1,
              cursor: selected_region === 'Custom' ? 'not-allowed' : 'pointer',
            }"
          >
            {{
              runtime.is_fetching[selected_region]
                ? "Fetching..."
                : "Fetch Market Data"
            }}
          </button>
        </div>
        <div
          class="control-panel-checkbox-row border-0!"
          v-if="selected_region !== 'Custom'"
        >
          <span>Auto fetch </span>
          <input
            type="checkbox"
            aria-label="Auto fetch"
            v-model="roster_config.auto_fetch"
            @change="() => start_fetch(selected_region)"
          />
        </div>
        <span
          v-if="selected_region !== 'Custom'"
          class="mt-3 text-xs text-(--text-muted)"
        >
          Prices typed here are temporary (the next fetch replaces them). Change
          your region to "Custom" to keep your own prices.
        </span>
      </div>

      <div v-if="roster_ids.length > 1" class="side-bar-item">
        <span class="text-nowrap"> Active Roster: </span>
        <select
          v-model="selected_roster_id"
          class="selector"
          aria-label="Active roster"
        >
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
        label-text="Convert owned T4 Roster & Tradable to T4.5 Serca mats"
        :tooltip-text="`${CONVERT_SUMMARY}. The result is roster bound; the leftover stays.`"
        @change-tier="
          convert_pooled(
            selected_roster_mats_owned,
            selected_tradable_mats_owned,
          )
        "
      />
    </template>

    <template #main>
      <BudgetGrid :selected_roster_id="selected_roster_id" />
    </template>
  </Sidebar>
</template>
