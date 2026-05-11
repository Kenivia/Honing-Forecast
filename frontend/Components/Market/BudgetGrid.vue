<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
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
import { GridConfig } from "@/Utils/GridStyling";

const roster_store = useRosterStore();
const { roster_config, active_roster_mats_owned, active_tradable_mats_owned } =
  storeToRefs(roster_store);

watchEffect(() => {
  const t4_price = input_column_to_num(
    roster_store.roster_config.mats_prices[0],
  );
  const serca_price = input_column_to_num(
    roster_store.roster_config.mats_prices[1],
  );
  roster_store.roster_config.effective_serca_price = ALL_LABELS[1].map(
    (_, index) => Math.min(t4_price[index] * 5, serca_price[index]),
  );
});

const t4_better = computed(() => {
  const t4_price = input_column_to_num(roster_config.value.mats_prices[0]);
  const serca_price = input_column_to_num(roster_config.value.mats_prices[1]);
  return ALL_LABELS[1].map(
    (_, index) => t4_price[index] * 5 < serca_price[index],
  );
});

const T4_indices_to_watch = SERCA_SYNC_MAP.map(({ T4_index }) => T4_index);

watch(
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

const grids = computed((): GridConfig[] => [
  {
    tier: 1,
    cols: "250px 120px 120px 130px",
    grid_row_span: `span ${ALL_LABELS[1].length + 1}`,

    rows: ALL_LABELS[1].map((label, row) => ({
      label,
      col: row in SERCA_TO_T4 ? 0 : 1,
      row:
        row in SERCA_TO_T4 ? (SERCA_TO_T4 as Record<number, number>)[row] : row,
    })),
  },
  {
    tier: 0,
    cols: "250px 120px 150px",
    grid_row_span: undefined,
    rows: ALL_LABELS[0].map((label, row) => ({
      label,
      col: 0,
      row: row,
    })),
  },
]);

function price_suffix(label: string, row: number): string {
  if (label === "Shards")
    return "x" + roster_config.value.selected_shard_bag_size.toString();
  if (BUNDLE_SIZE[row] > 1)
    return "x" + BUNDLE_SIZE[row].toLocaleString("en-US");
  return "";
}
</script>

<template>
  <div
    v-if="roster_config.market_rerender_trigger"
    class="flex w-max max-w-full flex-row flex-wrap gap-2"
  >
    <div
      v-for="grid in grids"
      :key="grid.tier"
      class="card-shell outer-grid"
      :style="{ '--grid-cols': grid.cols, gridRow: grid.grid_row_span }"
    >
      <div class="table-title-row border-b border-(--border-medium)">
        <span class="text-right text-(--graph-roster-color)"
          >Roster Bound Mats</span
        >
        <span class="text-left text-(--graph-tradable-color)"
          >Tradable Mats</span
        >
        <span class="text-left text-(--text-muted)">Market price</span>
        <span v-if="grid.tier == 1" class="text-left text-(--gold)"
          >Effective price</span
        >
      </div>

      <div class="card-body contents pt-0!">
        <div
          v-for="{ label, col, row } in grid.rows"
          :key="`roster-input-${grid.tier}-${label}`"
          class="mats-row"
        >
          <MaterialCell
            :input_column="active_roster_mats_owned[col]"
            :row="row"
            :label="label"
            :setter="
              (val) => {
                active_roster_mats_owned[col].data[row] = val;
              }
            "
            input_color="var(--graph-roster-color)"
            :hide_tick="true"
          />
          <MaterialCell
            :input_column="active_tradable_mats_owned[col]"
            :row="row"
            :setter="
              (val) => {
                active_tradable_mats_owned[col].data[row] = val;
              }
            "
            input_color="var(--graph-tradable-color)"
          />
          <MaterialCell
            :input_column="roster_config.mats_prices[col]"
            :row="row"
            :setter="
              (val) => {
                roster_config.mats_prices[col].data[row] = val;
              }
            "
            :suffix="price_suffix(label, row)"
            :input_width="70"
            input_color="var(--text-muted)"
          />
          <MaterialCell
            v-if="grid.tier == 1 && !SYNCED_LABELS.includes(label)"
            :input_column="roster_config.effective_serca_price"
            :row="row"
            :suffix="t4_better[row] ? 'Convert T4' : 'Buy Serca '"
            input_color="var(--gold)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.outer-grid {
  display: grid;
  grid-template-columns: var(--grid-cols);
  align-items: center;
  background: var(--bg-dark);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  width: max-content;
  height: max-content;
  row-gap: 0px;
}

.table-title-row,
.mats-row {
  display: grid;
  grid-column: 1 / -1;
  grid-template-columns: subgrid;
  align-items: center;
  border-bottom: 1px solid var(--border-muted);
  height: 42 px;
  min-height: 0;
  text-align: center;
}
</style>
