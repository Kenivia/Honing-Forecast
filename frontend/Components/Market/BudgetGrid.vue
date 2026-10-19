<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  bundle_key,
  convert_is_cheaper,
  effective_unit_prices,
  Material,
  SHARED_LABELS,
  TIER_MATERIALS,
  unit_prices,
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
const selected_bundles = computed(
  () => roster_config.value.selected_bundles[selected_region.value],
);

// per-unit prices, which is what the convert-or-buy comparison needs
const selected_unit_prices = computed(() =>
  unit_prices(
    input_column_to_num(selected_mats_prices.value),
    selected_bundles.value,
  ),
);
const effective_prices = computed(() =>
  effective_unit_prices(selected_unit_prices.value),
);

// the price cell edits the bundle the user has chosen for that material
const price_key = (mat: Material) =>
  bundle_key(mat.label, selected_bundles.value[mat.label]);

// a material sold in one bundle shows its size as a suffix; several get a dropdown
function price_suffix(mat: Material): string {
  const size = selected_bundles.value[mat.label];
  return mat.bundle_sizes.length > 1 || size <= 1
    ? ""
    : "x" + size.toLocaleString("en-US");
}

const grids = computed((): GridConfig[] => [
  {
    tier: 0,
    grid_template_columns: "250px 100px 150px", // market price a bit bigger to fit the x1mil silver suffix
    materials: TIER_MATERIALS[0],
  },
  {
    tier: 1,
    grid_template_columns: "250px 100px 120px 138px",
    materials: TIER_MATERIALS[1],
  },
]);
</script>

<template>
  <div class="flex w-max max-w-full flex-row flex-wrap justify-around gap-2">
    <div
      v-for="grid in grids"
      :key="grid.tier"
      class="card-shell outer-grid"
      role="group"
      :aria-label="grid.tier == 0 ? 'T4 materials' : 'Serca materials'"
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
          v-for="mat in grid.materials"
          :key="`roster-input-${grid.tier}-${mat.label}`"
          class="mats-row"
          role="group"
          :aria-label="mat.label"
        >
          <MaterialCell
            :input_column="selected_roster_mats_owned"
            :label="mat.label"
            :show_label="true"
            aria_name="Roster bound owned"
            :setter="
              (val) => {
                selected_roster_mats_owned.values[mat.label] = val;
              }
            "
            input_color="var(--roster)"
            :hide_tick="true"
          />
          <MaterialCell
            :input_column="selected_tradable_mats_owned"
            :label="mat.label"
            :setter="
              (val) => {
                selected_tradable_mats_owned.values[mat.label] = val;
              }
            "
            input_color="var(--tradable)"
            aria_name="Tradable owned"
            :input_width="100"
          />
          <div class="flex flex-row items-center">
            <MaterialCell
              :input_column="selected_mats_prices"
              :label="price_key(mat)"
              :setter="
                (val) => {
                  selected_mats_prices.values[price_key(mat)] = val;
                }
              "
              :suffix="price_suffix(mat)"
              aria_name="Market price"
              :input_width="70"
              input_color="var(--text-muted)"
              :justify_left="true"
            />
            <select
              v-if="mat.bundle_sizes.length > 1"
              v-model.number="selected_bundles[mat.label]"
              class="selector annotation ml-1! h-fit px-0!"
              :aria-label="`${mat.label} bundle size`"
            >
              <option
                v-for="size in mat.bundle_sizes"
                :key="size"
                :value="size"
              >
                x{{ size.toLocaleString("en-US") }}
              </option>
            </select>
          </div>
          <MaterialCell
            v-if="grid.tier == 1 && !SHARED_LABELS.includes(mat.label)"
            :input_column="effective_prices"
            :label="mat.label"
            :suffix="
              convert_is_cheaper(mat, selected_unit_prices)
                ? 'Convert T4'
                : 'Buy Serca '
            "
            input_color="var(--gold)"
            class="pr-2"
          />
        </div>
      </div>
    </div>
  </div>
</template>
