<script setup lang="ts">
import { useRosterStore as useRosterStore } from "@/Stores/RosterConfig";
import {
  ALL_LABELS,
  BUNDLE_SIZE,
  SERCA_SYNC_MAP,
  SERCA_TO_T4,
  SYNCED_LABELS,
} from "@/Utils/Constants";
import { storeToRefs } from "pinia";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import { computed, watch, watchEffect } from "vue";
import { input_column_to_num } from "@/Utils/InputColumn";
import { force_rerender } from "./MarketUtil";

const roster_store = useRosterStore();
const { roster_config, active_roster_mats_owned, active_tradable_mats_owned } =
  storeToRefs(roster_store);

watchEffect(() => {
  let t4_price = input_column_to_num(roster_store.roster_config.mats_prices[0]);
  let serca_price = input_column_to_num(
    roster_store.roster_config.mats_prices[1],
  );
  roster_store.roster_config.effective_serca_price = ALL_LABELS[1].map(
    (_, index) => Math.min(t4_price[index] * 5, serca_price[index]),
  );
});
const t4_better = computed(() => {
  let t4_price = input_column_to_num(roster_config.value.mats_prices[0]);
  let serca_price = input_column_to_num(roster_config.value.mats_prices[1]);
  return ALL_LABELS[1].map(
    (_, index) => t4_price[index] * 5 < serca_price[index],
  );
});

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
  <div
    v-if="roster_config.market_rerender_trigger"
    class="flex w-max max-w-full flex-row flex-wrap gap-2"
  >
    <div
      class="roster-inputs-serca card-shell card-body"
      :style="{ gridRow: `span ${String(ALL_LABELS[1].length + 1)}` }"
    >
      <div class="table-title-row border-b border-(--border-medium)">
        <span class="pr-1 text-right text-(--graph-roster-color)"
          >Roster Bound Mats</span
        >
        <span class="text-(--graph-tradable-color)">Tradable Mats</span>
        <span class="text-(--text-muted)">Market price</span>
        <span class="text-(--gold)">Effective price</span>
      </div>

      <div
        v-for="(label, row) in ALL_LABELS[1]"
        :key="`roster-input-serca-${label}`"
        class="mats-row"
      >
        <MaterialCell
          :input_column="active_roster_mats_owned[row in SERCA_TO_T4 ? 0 : 1]"
          :row="row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row"
          :label="label"
          :setter="
            (val) => {
              active_roster_mats_owned[row in SERCA_TO_T4 ? 0 : 1].data[
                row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row
              ] = val;
            }
          "
          input_color="var(--graph-roster-color)"
          :hide_tick="true"
        />
        <MaterialCell
          :input_column="active_tradable_mats_owned[row in SERCA_TO_T4 ? 0 : 1]"
          :row="row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row"
          :setter="
            (val) => {
              active_tradable_mats_owned[row in SERCA_TO_T4 ? 0 : 1].data[
                row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row
              ] = val;
            }
          "
          input_color="var(--graph-tradable-color)"
        />
        <MaterialCell
          :input_column="roster_config.mats_prices[row in SERCA_TO_T4 ? 0 : 1]"
          :row="row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row"
          :setter="
            (val) => {
              roster_config.mats_prices[row in SERCA_TO_T4 ? 0 : 1].data[
                row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row
              ] = val;
            }
          "
          :suffix="
            label === 'Shards'
              ? 'x' + roster_config.selected_shard_bag_size.toString()
              : BUNDLE_SIZE[row] > 1
                ? 'x' + BUNDLE_SIZE[row].toLocaleString('en-US')
                : ''
          "
          :input_width="70"
          input_color="var(--text-muted)"
        />
        <MaterialCell
          v-if="!SYNCED_LABELS.includes(label)"
          :input_column="roster_config.effective_serca_price"
          :row="row"
          :suffix="t4_better[row] ? 'Convert T4' : 'Buy Serca '"
          input_color="var(--gold)"
        />
      </div>
    </div>
    <div class="card-shell roster-inputs-tier-4 card-body">
      <div class="table-title-row border-b border-(--border-medium)">
        <span class="pr-1 text-right text-(--graph-roster-color)"
          >Roster Bound Mats</span
        >
        <span class="text-(--graph-tradable-color)">Tradable Mats</span>
        <span class="text-(--text-muted)">Market price</span>
      </div>

      <div
        v-for="(label, row) in ALL_LABELS[0]"
        :key="`roster-input-t4-${label}`"
        class="mats-row"
      >
        <MaterialCell
          :input_column="active_roster_mats_owned[0]"
          :row="row"
          :label="label"
          :setter="
            (val) => {
              active_roster_mats_owned[0].data[row] = val;
            }
          "
          input_color="var(--graph-roster-color)"
          :hide_tick="true"
        />
        <MaterialCell
          :input_column="active_tradable_mats_owned[0]"
          :row="row"
          :setter="
            (val) => {
              active_tradable_mats_owned[0].data[row] = val;
            }
          "
          input_color="var(--graph-tradable-color)"
        />
        <MaterialCell
          :input_column="roster_config.mats_prices[0]"
          :row="row"
          :setter="
            (val) => {
              roster_config.mats_prices[0].data[row] = val;
            }
          "
          :suffix="
            label === 'Shards'
              ? 'x' + roster_config.selected_shard_bag_size.toString()
              : BUNDLE_SIZE[row] > 1
                ? 'x' + BUNDLE_SIZE[row].toLocaleString('en-US')
                : ''
          "
          :input_width="70"
          input_color="var(--text-muted)"
        />
      </div>
    </div>
  </div>
</template>
<style scoped>
.roster-inputs-tier-4 {
  display: grid;
  grid-template-columns: 250px 120px 150px;
  align-items: center;
  background: var(--bg-dark);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  width: max-content;
  height: max-content;
}

.roster-inputs-serca {
  display: grid;
  grid-template-columns: 250px 120px 120px 130px;
  align-items: center;
  background: var(--bg-dark);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  width: max-content;
  height: max-content;
}

.roster-inputs-tier-4 .table-title-row,
.roster-inputs-tier-4 .mats-row {
  display: grid;
  grid-column: 1 / -1;
  grid-template-columns: 250px 120px 120px;
  align-items: center;
  border-bottom: 1px solid var(--border-muted);
  min-height: 0;
  height: 36px;
}

.roster-inputs-serca .table-title-row,
.roster-inputs-serca .mats-row {
  display: grid;
  grid-column: 1 / -1;
  grid-template-columns: 250px 120px 120px 120px;
  align-items: center;
  border-bottom: 1px solid var(--border-muted);
  min-height: 0;
  height: 36px;
}
</style>
