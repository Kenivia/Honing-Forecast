<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  ALL_LABELS,
  BUNDLE_SIZE,
  SERCA_SYNC_MAP,
  SERCA_TO_T4_INDICES,
  SYNCED_LABELS,
} from "@/Utils/Constants";
import { storeToRefs } from "pinia";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import { computed, watch, watchEffect } from "vue";
import { input_column_to_num } from "@/Utils/InputColumn";

import { GridConfig } from "@/Utils/GridStyling";

const roster_store = useRosterStore();
const { roster_config, active_roster_mats_owned, active_tradable_mats_owned } =
  storeToRefs(roster_store);

watchEffect(() => {
  const t4_price = input_column_to_num(roster_store.active_mats_prices[0]);
  const serca_price = input_column_to_num(roster_store.active_mats_prices[1]);
  roster_store.roster_config.effective_serca_price = ALL_LABELS[1].map(
    (_, index) => Math.min(t4_price[index] * 5, serca_price[index]),
  );
});

const t4_better = computed(() => {
  const t4_price = input_column_to_num(roster_store.active_mats_prices[0]);
  const serca_price = input_column_to_num(roster_store.active_mats_prices[1]);
  return ALL_LABELS[1].map(
    (_, index) => t4_price[index] * 5 < serca_price[index],
  );
});

const T4_indices_to_watch = SERCA_SYNC_MAP.map(({ T4_index }) => T4_index);

watch(
  () =>
    T4_indices_to_watch.flatMap((T4_index) => [
      roster_store.active_mats_prices[0].data[T4_index],
      roster_store.active_tradable_mats_owned[0].data[T4_index],
      roster_store.active_roster_mats_owned[0].data[T4_index],
    ]),
  () => {
    for (const { serca_index, T4_index } of SERCA_SYNC_MAP) {
      roster_store.active_mats_prices[1].data[serca_index] =
        roster_store.active_mats_prices[0].data[T4_index];
      roster_store.active_tradable_mats_owned[1].data[serca_index] =
        roster_store.active_tradable_mats_owned[0].data[T4_index];
      roster_store.active_roster_mats_owned[1].data[serca_index] =
        roster_store.active_roster_mats_owned[0].data[T4_index];
    }
  },
  { deep: false, immediate: true },
);

const grids = computed((): GridConfig[] => [
  {
    tier: 0,
    grid_template_columns: "250px 100px 150px", // market price a bit bigger to fit the x1mil silver suffix
    rows: ALL_LABELS[0].map((label, row) => ({
      label,
      col: 0,
      row: row,
    })),
  },
  {
    tier: 1,
    grid_template_columns: "250px 100px 120px 138px",

    rows: ALL_LABELS[1].map((label, row) => ({
      label,
      col: row in SERCA_TO_T4_INDICES ? 0 : 1,
      row:
        row in SERCA_TO_T4_INDICES
          ? (SERCA_TO_T4_INDICES as Record<number, number>)[row]
          : row,
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
  <div class="flex w-max max-w-full flex-row flex-wrap justify-around gap-2">
    <div
      v-for="grid in grids"
      :key="grid.tier"
      class="card-shell outer-grid"
      :style="{
        '--grid-cols': grid.grid_template_columns,
      }"
    >
      <div class="mats-row h-fit! border-b border-(--border-main) pb-0!">
        <span class="w-25 justify-self-end text-left text-(--roster)"
          >Roster Bound owned
        </span>
        <span class="text-left text-(--tradable)">Tradable owned </span>
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
            input_color="var(--roster)"
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
            input_color="var(--tradable)"
            :input_width="100"
          />
          <MaterialCell
            :input_column="roster_store.active_mats_prices[col]"
            :row="row"
            :setter="
              (val) => {
                roster_store.active_mats_prices[col].data[row] = val;
              }
            "
            :suffix="price_suffix(label, row)"
            :input_width="70"
            input_color="var(--text-muted)"
            :justify_left="true"
          />
          <MaterialCell
            v-if="grid.tier == 1 && !SYNCED_LABELS.includes(label)"
            :input_column="roster_config.effective_serca_price"
            :row="row"
            :suffix="t4_better[row] ? 'Convert T4' : 'Buy Serca '"
            input_color="var(--gold)"
            class="pr-2"
          />
        </div>
      </div>
    </div>
  </div>
</template>
