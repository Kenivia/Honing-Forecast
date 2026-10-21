<script setup lang="ts">
import {
  ALL_LABELS,
  JUICE_RANGES,
  MATERIALS,
  NUM_BASE_MATS,
  ANNOTATION_COLORS,
  ANNOTATION_POSITIONS,
  ANNOTATION_LABELS,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import { has_upgrades_in_range, metric_to_text } from "@/Utils/Helpers";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import MaterialGraph from "@/Components/Character/MaterialDist/MaterialGraph.vue";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { computed, ref } from "vue";
import { input_column_to_num } from "@/Utils/InputColumn";
import {
  start_all_workers,
  start_eval_hist,
} from "@/Components/Character/CharWorkerUtils";
import { GridConfig } from "@/Utils/GridStyling";
import { useMediaQuery } from "@vueuse/core";
import Popup from "@/Components/Common/Popup.vue";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { chest_gains } from "@/Utils/Chest";
import {
  annotations_shown,
  owned_lines,
  special_graph_data,
  t4_gains as t4_gains_of,
} from "./DistLines";
const runtime = useRuntimeStore();

const { active_profile } = storeToRefs(useRosterStore());
const {
  roster_config,
  active_roster_mats_owned,
  active_tradable_mats_owned,
  processed_chests,
} = storeToRefs(useRosterStore());
const histogram_result = computed(() => runtime.histogram.result);

// Rust keys every material result by label, so these are read straight through.
const tier = computed(() => active_profile.value.tier);
const set_special_budget = (val: string) => {
  active_profile.value.special_budget.values[
    SPECIAL_LEAP_LABELS[active_profile.value.tier]
  ] = val;
};
const zeroes = (): Record<string, number> =>
  Object.fromEntries(ALL_LABELS[tier.value].map((label) => [label, 0]));

// what the chests give under the chest states the histogram last evaluated
const gains = computed(() =>
  chest_gains(
    processed_chests.value,
    runtime.histogram.result?.state_bundle.chest_states,
  ),
);
const gain = (label: string, band: number) => gains.value[label]?.[band] ?? 0;

// the three owned columns, as numbers
const owned = computed(
  () =>
    [
      active_profile.value.bound_budgets,
      active_roster_mats_owned.value,
      active_tradable_mats_owned.value,
    ].map((column) => input_column_to_num(column)) as [
      Record<string, number>,
      Record<string, number>,
      Record<string, number>,
    ],
);
// what a Serca character's T4 adds to each band, as Rust counts it
const t4_gains = computed(() =>
  t4_gains_of(
    tier.value,
    owned.value,
    active_profile.value.bound_budgets.enabled,
    input_column_to_num(active_profile.value.special_budget),
  ),
);
const t4_gain = (label: string, band: number) =>
  t4_gains.value[label]?.[band] ?? 0;
// shown only when there is any
const t4_line = (amount: number) =>
  amount > 0 ? [{ amount, text: "from T4" }] : [];

// This is average mats cost (not gold)
const average_breakdown = computed(
  () => runtime.histogram.result?.avg_used ?? zeroes(),
);
// plan 0 is always ALL_WORTHLESS, so this is gold actually handed to the market
const gold_breakdown = computed(() => {
  const per_mat: Record<string, number> | undefined =
    runtime.histogram.result?.gold_per_plan[0];
  if (per_mat === undefined) return zeroes();
  return Object.fromEntries(
    Object.entries(per_mat).map(([label, x]) => [label, x >= 0 ? 0 : -x]),
  );
});
const histogram_chances = computed(() => {
  const per_mat: Record<string, number[]> | undefined =
    runtime.histogram.result?.chance_within;
  if (per_mat === undefined) return zeroes();
  return Object.fromEntries(
    Object.entries(per_mat).map(([label, bands]) => [
      label,
      bands[active_profile.value.chance_band],
    ]),
  );
});

const visibleRows = computed(() =>
  ALL_LABELS[active_profile.value.tier]
    .map((label, row) => ({ label, row, color: MATERIALS[label].color }))
    .filter(({ label, row }) => {
      if (row < NUM_BASE_MATS) return true;
      if (label === "Lava's Breath" || label === "Glacier's Breath")
        return true;
      if (roster_config.value.show_all_rows) return true;
      if (average_breakdown.value[label] > 0.0) return true;
      // a juice row only shows when an upgrade in its range is wanted
      const range = JUICE_RANGES[label];
      return (
        active_profile.value.tier === 0 &&
        range !== undefined &&
        has_upgrades_in_range(active_profile.value, ...range)
      );
    }),
);

const tickbox_tooltip = `Untick the box if you don't plan on buying that material from market.`;
const marginal_tooltip = `Average tradable gold saved per extra Char-Bound unit, for one more tap's worth of it (ten with x10 streak length).`;

// from the last finished optimizer result; faded while a newer one is on its way
const marginal = computed<Record<string, number>>(
  () => runtime.marginal.result ?? {},
);
const marginal_stale = computed(
  () =>
    runtime.marginal.status === "busy" || runtime.optimizer.status === "busy",
);
// <span style="color:var(--text-muted)">(it also disable selling this mat)</span>`

const total_market_gold_text = "Average tradable gold spent:";
const total_market_gold_suffix = "(raw  +  buying needed mats & juice)";
const total_tradable_gold_text = "Avg value of leftover mats:";
const total_tradable_gold_suffix = "";

// Plan 0 credits nothing, so it is the gross spend; plan 1 is the user's, which is what
// the optimizer minimises. The gap between them is what the leftovers are worth.
const total_gold = computed(() => runtime.histogram.result?.metric_per_plan[0]);
const leftover_credit = computed(
  () =>
    runtime.histogram.result?.metric_per_plan[0] -
    runtime.histogram.result?.metric_per_plan[1],
);
const any_credited = computed(() =>
  active_profile.value.band_values.some((v) => v !== "Worthless"),
);

// parallel to chance_band
const CHANCE_TEXTS = ["Char-Bound", "Roster-Bound", "Tradable"].map(
  (band) => `Chance to succeed all upgrades within ${band} material`,
);
const CHANCE_COLORS = ["var(--bound)", "var(--roster)", "var(--tradable)"];

const selected_chance_color = computed(
  () => CHANCE_COLORS[active_profile.value.chance_band],
);

function change_chance_band(event: Event) {
  active_profile.value.chance_band = Number(
    (event.target as HTMLSelectElement).value,
  );
  start_eval_hist();
}

const enabled_annotations = computed(() =>
  annotations_shown(active_profile.value.chance_band),
);

const annotation_values = computed(() => {
  const traded = [
    active_profile.value.bound_budgets,
    active_roster_mats_owned.value,
    active_tradable_mats_owned.value,
  ].map((column) => column.enabled);
  const out: Record<string, (number | null)[]> = {};
  for (const label of ALL_LABELS[tier.value]) {
    out[label] = owned_lines(
      average_breakdown.value[label],
      owned.value.map((column) => column[label]),
      [0, 1, 2].map((band) => gain(label, band) + t4_gain(label, band)),
      traded.map((enabled) => enabled[label]),
    ).filter((_, i) => enabled_annotations.value[i]);
  }
  return out;
});

const special_data = computed(() => {
  const bundle = runtime.histogram.result?.state_bundle;
  return bundle ? special_graph_data(bundle) : null;
});

function hover_annotation(x, _y, cy, material_type, color, is_last): string {
  let place = is_last
    ? 3
    : Math.min(
        10,
        Math.max(
          3,
          Math.ceil(
            cy < 0.5
              ? Math.min(3, Math.abs(Math.log10(cy)))
              : Math.abs(Math.log10(1 - cy)),
          ),
        ),
      );

  return `<b style="color: white;">${(cy * 100).toPrecision(place)}% </b> chance to use <br> ≤ <b style="color: ${color};"> ${Math.ceil(x).toLocaleString("en-US")} </b> ${material_type} `;
}
function special_hover_annotation(x, _y, cy, material_type, color): string {
  let place = Math.min(
    10,
    Math.max(
      Math.ceil(
        cy < 0.5
          ? Math.min(3, Math.abs(Math.log10(cy)))
          : Math.abs(Math.log10(1 - cy)),
      ),
      3,
    ),
  );
  return `<b style="color: white;">${(cy * 100).toPrecision(place)}% </b> chance to free tap <br> at least <b style="color: ${color};"> ${x + 1} </b> piece`;
}

const show_special_guide = ref(false);

const grid: GridConfig = {
  grid_template_columns:
    "minmax(260px, 320px) minmax(90px, 110px) minmax(70px, 90px) minmax(110px, 120px) minmax(110px, 120px) 350px",
};

const is924Narrow = useMediaQuery("(max-width: 924px)"); // this turns out to be the width where the checkboxes overlap the labels
</script>

<template>
  <div class="card-shell">
    <div class="card-header">
      <div class="card-title">Costs distribution</div>
    </div>
    <div
      class="card-body outer-grid pt-0! pb-2!"
      :style="{
        '--grid-cols': grid.grid_template_columns,
      }"
    >
      <div class="mats-row h-fit! items-end! border-b-(--border-main)!">
        <div class="flex flex-row justify-between">
          <QuestionMark :text="tickbox_tooltip" class="mb-1" />
          <span class="mr-17.5 w-25 text-left text-(--bound)">Bound Mats</span>
        </div>
        <div class="flex flex-row items-end justify-end gap-1">
          <QuestionMark :text="marginal_tooltip" class="mb-1" />
          <span class="text-right text-(--gold)">Est. gold value</span>
        </div>
        <select
          aria-label="Chance column"
          class="selector -mr-4! ml-4!"
          :value="active_profile.chance_band"
          :style="{
            color: selected_chance_color,
          }"
          @change="change_chance_band"
        >
          <option
            v-for="(text, band) in CHANCE_TEXTS"
            :key="band"
            :value="band"
          >
            {{ text }}
          </option>
        </select>
        <span class="text-right text-(--average)">Average</span>
        <div class="flex flex-row content-end">
          <span class="w-full basis-full text-right text-nowrap text-(--gold)"
            >Avg Gold used</span
          >
          <span
            class="w-0 basis-0 origin-right transform-[translateY(4px)] pl-px text-left text-xs text-(--text-very-muted)"
            >(tradable)</span
          >
        </div>

        <!-- <span class="text-(--text-muted)">Hover graph for details!</span> -->
        <label class="flex flex-row justify-end gap-1">
          <input v-model="roster_config.cumulative_graph" type="checkbox" />
          <span>Cumulative graph</span>
        </label>
        <!-- <span v-if="customLeftovers">Left</span> -->
      </div>
      <div v-if="runtime.histogram.result" class="contents">
        <div
          v-for="{ label, row, color } in visibleRows"
          :key="`graph-${label}`"
          class="mats-row"
          role="group"
          :aria-label="label"
          :class="{
            disabled: !active_profile.bound_budgets.enabled[label],
          }"
        >
          <MaterialCell
            :input_column="active_profile.bound_budgets"
            :label="label"
            :show_label="true"
            aria_name="Bound owned"
            :input_color="'--bound'"
            :setter="
              (val) => {
                active_profile.bound_budgets.values[label] = val;
              }
            "
            :hide_tick="row >= NUM_BASE_MATS"
            :callback="() => start_all_workers()"
            :hide_label="is924Narrow && row < NUM_BASE_MATS"
            :bound_label="true"
            :plus="[
              {
                amount: gain(label, active_profile.chance_band),
                text: 'from chests',
              },
              ...t4_line(t4_gain(label, active_profile.chance_band)),
            ]"
          />
          <MaterialCell
            v-if="marginal[label] !== undefined"
            :input_column="marginal"
            :label="label"
            :input_color="'--gold'"
            :class="{ 'opacity-50': marginal_stale }"
          />
          <div v-else></div>
          <MaterialCell
            :input_column="histogram_chances"
            :label="label"
            :input_color="selected_chance_color"
            :is_percentage="true"
          />
          <MaterialCell
            :input_column="average_breakdown"
            :label="label"
            :input_color="'--average'"
          />
          <MaterialCell
            :input_column="gold_breakdown"
            :label="label"
            :input_color="'--gold'"
          />
          <MaterialGraph
            :data="histogram_result?.cum_percentiles?.[label] ?? null"
            :material-label="label"
            :graph-color="color"
            :cumulative="roster_config.cumulative_graph"
            :annotations="annotation_values[label]"
            :annotationColors="
              ANNOTATION_COLORS.filter((_, i) => enabled_annotations[i])
            "
            :annotation-positions="
              ANNOTATION_POSITIONS.filter((_, i) => enabled_annotations[i])
            "
            :annotationLabels="
              ANNOTATION_LABELS.filter((_, i) => enabled_annotations[i])
            "
            :tooltip-text-fn="hover_annotation"
          />
        </div>

        <div class="mats-row">
          <MaterialCell
            :input_column="active_profile.special_budget"
            :label="SPECIAL_LEAP_LABELS[active_profile.tier]"
            :show_label="true"
            :setter="set_special_budget"
            :hide_tick="true"
            aria_name="Special leaps owned"
            :callback="() => start_all_workers()"
            :plus="[
              {
                amount: (
                  gains[SPECIAL_LEAP_LABELS[active_profile.tier]] ?? []
                ).reduce((a, b) => a + b, 0),
                text: 'from chests',
              },
              ...t4_line(t4_gain(SPECIAL_LEAP_LABELS[active_profile.tier], 0)),
            ]"
          ></MaterialCell>
          <span
            class="h-max text-xs leading-3 text-(--free-tap) underline hover:text-(--free-tap-bright)"
            @click="() => (show_special_guide = true)"
            >Should I use in T4 or convert?</span
          >
          <MaterialGraph
            :data="special_data"
            :material-label="'Special'"
            :graph-color="'--free-tap'"
            :cumulative="roster_config.cumulative_graph"
            :tooltip-text-fn="special_hover_annotation"
            :max-yoverride="1"
            style="grid-column: span 4"
            :empty_message="'No normal honing available'"
            :upside_down="true"
          />
        </div>
      </div>
    </div>
    <div class="metric-container">
      <div class="metric-label text-(--gold)">
        {{ total_market_gold_text }}
      </div>
      <div class="metric-result-container">
        <span class="metric-result text-(--gold)">
          {{ metric_to_text(total_gold) }}
        </span>
        <span class="metric-result-suffix">
          {{ total_market_gold_suffix }}
        </span>
      </div>
    </div>
    <div v-if="any_credited" class="metric-container">
      <div class="metric-label text-(--text-muted)">
        {{ total_tradable_gold_text }}
      </div>
      <div class="metric-result-container">
        <span class="metric-result text-(--text-muted)">
          {{ metric_to_text(leftover_credit) }}
        </span>
        <span class="metric-result-suffix">
          {{ total_tradable_gold_suffix }}
        </span>
      </div>
    </div>

    <Popup
      :show_popup="show_special_guide"
      @close_popup="show_special_guide = false"
    >
      <span style="font-size: 30px; color: var(--text-bright)">
        Save Special Leaps and convert to Serca, unless you are tapping +25
      </span>
      <span style="font-size: 20px; color: var(--text-main)">
        If you're not +20 yet, use it in T4.
      </span>
      <img src="/Special convert chart.png" alt="Special convert chart" />
    </Popup>
  </div>
  <!-- <span>
    The above results assumes that you follow the optimal
    <RouterLink
      class="metric-label"
      style="text-decoration: underline"
      :to="{
        name: 'instructions',
        params: { characterName: active_profile.char_name },
      }"
    >
      Taps Instructions
    </RouterLink>
  </span> -->
</template>
<style scoped>
.metric-container {
  margin-left: auto;
  margin-right: auto;
  display: flex;
  width: 100%;
  min-width: 100%;
  flex-direction: row;
  flex-wrap: nowrap;
  align-content: center;
  gap: 0.25rem;
  margin-bottom: 0.5rem;
}

/* trying to align the gold value with the gold breadown column, kinda scuffed but works i tihnk*/
.metric-label {
  flex-shrink: 1;
  flex-basis: 52.9%; /* (320+90+120+16)/1032  */
  text-align: right;
  font-size: 1.25rem;
  line-height: 1.75rem;
}
/* aligning for the non-squished cases */
.metric-result-container {
  display: flex;
  min-width: calc(100% - 52.9%);
  flex-shrink: 1;
  flex-basis: calc(100% - 52.9%);
  flex-wrap: wrap;
  align-content: center;
}

.metric-result {
  flex-shrink: 1;
  font-size: 1.25rem;
  line-height: 1.75rem;
  min-width: 120px;
  text-align: right;
}

.metric-result-suffix {
  width: max-content;
  max-width: 100%;
  flex-shrink: 1;
  align-self: flex-end;
  font-size: 0.75rem;
  line-height: 1rem;
  text-wrap: wrap;
  color: var(--text-very-muted);
  transform: translateY(-0.25rem);
  padding-left: 1px;
}
</style>
