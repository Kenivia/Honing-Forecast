<script setup lang="ts">
import { default_prices, useRosterStore } from "@/Stores/RosterConfig";
import {
  bundle_key,
  convert_is_cheaper,
  effective_unit_prices,
  Material,
  TIER_MATERIALS,
  unit_prices,
} from "@/Utils/Constants";
import { storeToRefs } from "pinia";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import HoverTip from "@/Components/Common/HoverTip.vue";
import { computed } from "vue";
import { formatDate, useNow } from "@vueuse/core";
import { parse_sources } from "@/Utils/MarketDataFetcher";
import { input_column_to_num } from "@/Utils/InputColumn";
import { GridConfig } from "@/Utils/GridStyling";

const props = defineProps<{
  selected_roster_id: string;
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

// Under each price: how old the fetched price is and where it came from, else whether it is the
// region's default or the user's own. Decided by value alone. Gold and Silver have no market.
const now = useNow({ interval: 30_000 });
const price_nums = computed(() =>
  input_column_to_num(selected_mats_prices.value, true),
);
const default_nums = computed(() =>
  input_column_to_num(default_prices(selected_region.value), true),
);
const sources = computed(() =>
  parse_sources(
    roster_config.value.latest_market_data[selected_region.value]?.[1] ?? [],
  ),
);
const NO_MARKET = ["Gold", "Silver"];
const SOURCE_NAMES = {
  "loa-buddy":
    '<a href="https://loa-buddy.pages.dev" target="_blank" rel="noopener" class="link">loa-buddy</a>',
  hf: "Honing Forecast's own scanner",
};

function detailed_tip(source: keyof typeof SOURCE_NAMES, ms: number): string {
  const mins = Math.max(0, Math.floor((now.value.getTime() - ms) / 60_000));
  const plural = (n: number, unit: string) =>
    `${n} ${unit}${n === 1 ? "" : "s"}`;
  const days = Math.floor(mins / (24 * 60));
  const hours = Math.floor((mins / 60) % 24);
  const span = days
    ? `${plural(days, "day")} ${plural(hours, "hour")}`
    : hours
      ? `${plural(hours, "hour")} ${plural(mins % 60, "min")}`
      : plural(mins, "min");
  return `Price from ${SOURCE_NAMES[source]} scanned on ${formatDate(new Date(ms), "MMM Do HH:mm")} (${span} ago)`;
}

function ago(ms: number): string {
  const mins = Math.floor((now.value.getTime() - ms) / 60_000);
  const plural = (n: number, unit: string) =>
    `${n} ${unit}${n === 1 ? "" : "s"} ago`;
  if (mins < 1) return "just now";
  if (mins < 60) return plural(mins, "min");
  if (mins < 24 * 60) return plural(Math.floor(mins / 60), "hour");
  return plural(Math.floor(mins / (24 * 60)), "day");
}

function freshness(mat: Material): { text: string; tip?: string } | null {
  if (NO_MARKET.includes(mat.label)) return null;
  const key = price_key(mat);
  const value = price_nums.value[key];
  const fetched = sources.value[key];
  if (fetched?.price === value) {
    return {
      text: ago(fetched.at),
      tip: detailed_tip(fetched.source, fetched.at),
    };
  }
  if (value === default_nums.value[key]) {
    return {
      text: "Default",
      tip: `This item hasn't appear on the market in any scan yet`,
    };
  }
  return {
    text: "Manual",
    // tip: ``,
  };
}

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
    grid_template_columns: "250px 100px 150px 138px",
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
              aria_name="Market price"
              :input_width="70"
              input_color="var(--text-muted)"
              :justify_left="true"
            />
            <div class="ml-1 flex flex-col items-start gap-1">
              <select
                v-if="mat.bundle_sizes.length > 1"
                v-model.number="selected_bundles[mat.label]"
                class="selector annotation h-fit px-0! py-0!"
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
              <span v-else-if="price_suffix(mat)" class="annotation">{{
                price_suffix(mat)
              }}</span>
              <HoverTip
                v-if="freshness(mat)?.tip"
                :text="null"
                :sticky="true"
                placement="right"
              >
                <span
                  class="annotation whitespace-nowrap underline decoration-dotted underline-offset-2"
                  >{{ freshness(mat)!.text }}</span
                >
                <template #tip><span v-html="freshness(mat)!.tip" /></template>
              </HoverTip>
              <span
                v-else-if="freshness(mat)"
                class="annotation whitespace-nowrap"
                >{{ freshness(mat)!.text }}</span
              >
            </div>
          </div>
          <MaterialCell
            v-if="mat.from"
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
