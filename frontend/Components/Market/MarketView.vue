<script setup lang="ts">
import { useRosterStore as useRosterStore } from "@/Stores/RosterConfig";
import { ALL_LABELS, SERCA_SYNC_MAP, SYNCED_LABELS } from "@/Utils/Constants";
import { storeToRefs } from "pinia";
import { computed, watch, watchEffect } from "vue";
import TierConvertButton from "../Common/TierConvertButton.vue";
import { fetch_callback, useTimedFetch } from "@/Utils/MarketDataFetcher";
import { input_column_to_num, parse_input } from "@/Utils/InputColumn";
import Sidebar from "../Common/Sidebar.vue";
import BudgetGrid from "./BudgetGrid.vue";
import { force_rerender } from "./MarketUtil";

const roster_store = useRosterStore();
const {
  roster_config,
  active_roster_mats_owned,
  active_tradable_mats_owned,
  all_profiles,
  roster_ids,
  active_profile,
} = storeToRefs(roster_store);

const { disabled, start_fetch } = useTimedFetch((result, selected, price) => {
  fetch_callback(result, selected, price);
  force_rerender();
});

function convert_roster_mats_to_serca() {
  console.log("triggerd");
  for (let serca_index = 0; serca_index < ALL_LABELS[1].length; serca_index++) {
    if (!SYNCED_LABELS.includes(ALL_LABELS[1][serca_index])) {
      let T4_index = ALL_LABELS[0].findIndex(
        (x) => x == ALL_LABELS[1][serca_index].replace("Serca ", ""),
      );

      // all become roster bound
      active_roster_mats_owned.value[1].data[serca_index] = (
        input_column_to_num(active_roster_mats_owned.value[1])[T4_index] +
        parse_input(
          active_tradable_mats_owned.value[0],
          T4_index,
          String(
            input_column_to_num(active_tradable_mats_owned.value[0])[T4_index] *
              0.2,
          ),
        )
      ).toLocaleString();
      active_tradable_mats_owned.value[0].data[T4_index] = "0";
      active_roster_mats_owned.value[1].data[serca_index] = (
        input_column_to_num(active_roster_mats_owned.value[1])[T4_index] +
        parse_input(
          active_roster_mats_owned.value[0],
          T4_index,
          String(
            input_column_to_num(active_roster_mats_owned.value[0])[T4_index] *
              0.2,
          ),
        )
      ).toLocaleString();
      active_roster_mats_owned.value[0].data[T4_index] = "0";
    }
  }
  force_rerender();
}
watchEffect(() => {
  let t4_price = input_column_to_num(roster_store.roster_config.mats_prices[0]);
  let serca_price = input_column_to_num(
    roster_store.roster_config.mats_prices[1],
  );
  roster_store.roster_config.effective_serca_price = ALL_LABELS[1].map(
    (_, index) => Math.min(t4_price[index] * 5, serca_price[index]),
  );
});

function find_representative(): Record<string, number> {
  let out = {};
  let seen = {};
  let roster_index = 1;
  for (const [profile_index, profile] of all_profiles.value.entries()) {
    if (!Object.hasOwn(seen, profile.roster_id)) {
      seen[profile.roster_id] = roster_index;
      let name = "Roster " + String(roster_index);
      out[name] = profile_index;
      roster_index += 1;
    }
  }
  return out;
}
const representative_profile_indices = computed(find_representative);
const selected_roster = computed(() => {
  let out = Object.entries(representative_profile_indices.value).find(
    ([, v]) =>
      all_profiles.value[v].roster_id === active_profile.value.roster_id,
  )[0];
  // console.log(out)
  return out;
});
function change_roster(event) {
  // console.log(representative_profile_indices.value[event.target.value])
  roster_store.switchProfile(
    representative_profile_indices.value[event.target.value],
  );
  force_rerender();
}

const T4_indices_to_watch = SERCA_SYNC_MAP.map(({ T4_index }) => T4_index);

watch(
  // one way sync from T4 to Serca, the ui modifies the T4 copy
  () =>
    T4_indices_to_watch.flatMap((T4_index) => [
      roster_store.roster_config.mats_prices[0].data[T4_index],
      roster_store.active_tradable_mats_owned[0].data[T4_index],
      roster_store.active_roster_mats_owned[0].data[T4_index],
    ]),
  () => {
    for (const { serca_index, T4_index } of SERCA_SYNC_MAP) {
      roster_config.value.mats_prices[1].data[serca_index] =
        roster_config.value.mats_prices[0].data[T4_index];
      roster_store.active_tradable_mats_owned[1].data[serca_index] =
        roster_store.active_tradable_mats_owned[0].data[T4_index];
      roster_store.active_roster_mats_owned[1].data[serca_index] =
        roster_store.active_roster_mats_owned[0].data[T4_index];
    }
    force_rerender();
  },
  { deep: false, immediate: true },
);
</script>

<template>
  <Sidebar :width="1315" header="Market & mats">
    <template #sidebar>
      <div class="side-bar-item">
        <div class="flex flex-row items-center justify-around gap-3">
          <select
            v-model="roster_config.region"
            @change="
              () => {
                start_fetch(roster_config.region, true);
              }
            "
            class="selector"
          >
            <option>NAE</option>
            <option>EUC</option>
          </select>
          <span>
            {{
              !disabled &&
              roster_config.latest_market_data &&
              !roster_config.market_fetch_failed
                ? "✅"
                : disabled
                  ? ""
                  : "Failed"
            }}
          </span>
        </div>
        <button
          :disabled="disabled"
          @click="() => start_fetch(roster_config.region, true)"
          class="generic-button"
        >
          {{ !disabled ? "Fetch Market Data" : "Fetching..." }}
        </button>
      </div>

      <div class="side-bar-item">
        <div class="side-bar-item">
          <label class="text-nowrap">Shard bag size:</label>
          <select
            v-model.number="roster_config.selected_shard_bag_size"
            class="selector"
          >
            <option value="1000">x1000</option>
            <option value="2000">x2000</option>
            <option value="3000">x3000</option>
          </select>
        </div>
        <label class="text-(--text-muted)"
          >(Best one will be auto selected)</label
        >
      </div>

      <div v-if="roster_ids.length > 1" class="side-bar-item">
        <span class="text-nowrap"> Active Roster: </span>
        <select
          :value="selected_roster"
          class="selector"
          @change="change_roster"
        >
          <option
            v-for="(profile_index, name) in representative_profile_indices"
            :value="name"
            :key="profile_index"
          >
            {{ name }}
          </option>
        </select>
      </div>
      <TierConvertButton
        label-text="Convert owned T4 Roster & Tradable to T4.5 Serca mats (5 to 1 ratio)"
        tooltip-text="Red, Blue, and Leaps (not abidos)"
        @change-tier="convert_roster_mats_to_serca"
      ></TierConvertButton>
    </template>

    <template #main> <BudgetGrid /> </template>
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
