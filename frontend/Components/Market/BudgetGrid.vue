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
import { computed } from "vue";
import { input_column_to_num } from "@/Utils/InputColumn";
import { GridConfig } from "@/Utils/GridStyling";

const props = defineProps<{
  selected_roster_id: number;
}>();
const roster_store = useRosterStore();
const { roster_config } = storeToRefs(roster_store);

const selected_roster_mats_owned = computed(
  () => roster_config.value.roster_mats_owned[props.selected_roster_id],
);
const selected_tradable_mats_owned = computed(
  () => roster_config.value.tradable_mats_owned[props.selected_roster_id],
);
const selected_region = computed(
  () => roster_config.value.all_regions[props.selected_roster_id],
);
const selected_mats_prices = computed(
  () => roster_config.value.mats_prices[selected_region.value],
);

const t4_better = computed(() => {
  const t4_price = input_column_to_num(selected_mats_prices.value[0]);
  const serca_price = input_column_to_num(selected_mats_prices.value[1]);
  return ALL_LABELS[1].map(
    (_, index) => t4_price[index] * 5 < serca_price[index],
  );
});

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

function price_suffix(
  label: string,
  row: number,
  // region: MarketRegions,
): string {
  if (label === "Shards") return ""; //"x" + roster_config.value.selected_shard_bag_size[region].toString();
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
          :key="`roster-input-${grid.tier}-${row === 3 ? 'shard' + roster_config.shard_infos[selected_region].selected.toLocaleString() : label}`"
          class="mats-row"
        >
          <MaterialCell
            :input_column="selected_roster_mats_owned[col]"
            :row="row"
            :label="label"
            :setter="
              (val) => {
                selected_roster_mats_owned[col].data[row] = val;
              }
            "
            input_color="var(--roster)"
            :hide_tick="true"
          />
          <MaterialCell
            :input_column="selected_tradable_mats_owned[col]"
            :row="row"
            :setter="
              (val) => {
                selected_tradable_mats_owned[col].data[row] = val;
              }
            "
            input_color="var(--tradable)"
            :input_width="100"
          />
          <div class="flex flex-row items-center">
            <MaterialCell
              :input_column="
                row === 3
                  ? roster_config.shard_infos[selected_region].prices[
                      roster_config.shard_infos[selected_region].selected
                    ]
                  : selected_mats_prices[col]
              "
              :row="row === 3 ? 0 : row"
              :setter="
                (val) => {
                  row === 3
                    ? (roster_config.shard_infos[selected_region].prices[
                        roster_config.shard_infos[selected_region].selected
                      ].data[0] = val)
                    : (selected_mats_prices[col].data[row] = val);
                }
              "
              :suffix="price_suffix(label, row)"
              :input_width="70"
              input_color="var(--text-muted)"
              :justify_left="true"
            />
            <select
              v-if="row === 3"
              v-model.number="
                roster_config.shard_infos[selected_region].selected
              "
              class="selector annotation ml-1! h-fit px-0!"
            >
              <option :value="1000">x1000</option>
              <option :value="2000">x2000</option>
              <option :value="3000">x3000</option>
            </select>
          </div>
          <MaterialCell
            v-if="grid.tier == 1 && !SYNCED_LABELS.includes(label)"
            :input_column="roster_store.effective_serca_price"
            :row="row"
            :suffix="t4_better[row] ? 'Convert T4' : 'Buy Serca '"
            input_color="var(--gold)"
            class="pr-2"
          />

          <!-- <label class="text-nowrap">Shard bag size:</label> -->
        </div>
      </div>
    </div>
  </div>
</template>
