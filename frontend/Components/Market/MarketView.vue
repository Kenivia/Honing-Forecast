<script setup lang="ts">
import { useRosterStore as useRosterStore } from "@/Stores/RosterConfig";
import { ALL_LABELS, SERCA_SYNC_MAP, SYNCED_LABELS } from "@/Utils/Constants";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import TierConvertButton from "../Common/TierConvertButton.vue";
import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher";
import { input_column_to_num, parse_input } from "@/Utils/InputColumn";
import Sidebar from "../Common/Sidebar.vue";
import BudgetGrid from "./BudgetGrid.vue";
import RegionSelector from "../Common/RegionSelector.vue";

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
const selected_mats_prices = computed(
  () => roster_config.value.mats_prices[selected_region.value],
);
function convert_roster_mats_to_serca() {
  for (let serca_index = 0; serca_index < ALL_LABELS[1].length; serca_index++) {
    if (!SYNCED_LABELS.includes(ALL_LABELS[1][serca_index])) {
      let T4_index = ALL_LABELS[0].findIndex(
        (x) => x == ALL_LABELS[1][serca_index].replace("Serca ", ""),
      );

      // all become roster bound
      selected_roster_mats_owned.value[1].data[serca_index] = (
        input_column_to_num(selected_roster_mats_owned.value[1])[T4_index] +
        parse_input(
          selected_tradable_mats_owned.value[0],
          T4_index,
          String(
            input_column_to_num(selected_tradable_mats_owned.value[0])[
              T4_index
            ] * 0.2,
          ),
        )
      ).toLocaleString();
      selected_tradable_mats_owned.value[0].data[T4_index] = "0";
      selected_roster_mats_owned.value[1].data[serca_index] = (
        input_column_to_num(selected_roster_mats_owned.value[1])[T4_index] +
        parse_input(
          selected_roster_mats_owned.value[0],
          T4_index,
          String(
            input_column_to_num(selected_roster_mats_owned.value[0])[T4_index] *
              0.2,
          ),
        )
      ).toLocaleString();
      selected_roster_mats_owned.value[0].data[T4_index] = "0";
    }
  }
}

const T4_indices_to_watch = SERCA_SYNC_MAP.map(({ T4_index }) => T4_index);

watch(
  // one way sync from T4 to Serca, the ui modifies the T4 copy
  () =>
    T4_indices_to_watch.flatMap((T4_index) => [
      selected_mats_prices.value[0].data[T4_index],
      selected_tradable_mats_owned.value[0].data[T4_index],
      selected_roster_mats_owned.value[0].data[T4_index],
    ]),
  () => {
    for (const { serca_index, T4_index } of SERCA_SYNC_MAP) {
      selected_mats_prices.value[1].data[serca_index] =
        selected_mats_prices.value[0].data[T4_index];
      selected_tradable_mats_owned.value[1].data[serca_index] =
        selected_tradable_mats_owned.value[0].data[T4_index];
      selected_roster_mats_owned.value[1].data[serca_index] =
        selected_roster_mats_owned.value[0].data[T4_index];
    }
  },
  { deep: false, immediate: true },
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
              !roster_config.is_fetching && !roster_config.market_fetch_failed
                ? "✅"
                : roster_config.is_fetching
                  ? ""
                  : "Failed"
            }}
          </span>
          <button
            :disabled="
              roster_config.is_fetching || selected_region === 'Custom'
            "
            @click="() => start_fetch(selected_region, true)"
            class="generic-button mx-3! w-max!"
            :style="{
              opacity: selected_region === 'Custom' ? 0.5 : 1,
              cursor: selected_region === 'Custom' ? 'not-allowed' : 'pointer',
            }"
          >
            {{
              !roster_config.is_fetching ? "Fetch Market Data" : "Fetching..."
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
