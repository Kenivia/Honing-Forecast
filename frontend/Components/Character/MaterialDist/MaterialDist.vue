<script setup lang="ts">
import {
  ALL_LABELS,
  JUICE_RANGES,
  MATERIALS,
  NUM_BASE_MATS,
  ANNOTATION_COLORS,
  ANNOTATION_POSITIONS,
  ANNOTATION_LABELS,
  by_label,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import { TreatmentPlan } from "@/Stores/CharacterProfile";
import { has_upgrades_in_range, metric_to_text } from "@/Utils/Helpers";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import MaterialGraph from "@/Components/Character/MaterialDist/MaterialGraph.vue";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { computed, onMounted, ref, watchEffect } from "vue";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { input_column_to_num } from "@/Utils/InputColumn";
import { start_all_workers } from "@/Components/Character/CharWorkerUtils";
import { WasmOp } from "@/WasmInterface/WasmWorker";
import { GridConfig } from "@/Utils/GridStyling";
import { useMediaIsNarrow } from "@/Utils/WindowSize";
import Popup from "@/Components/Common/Popup.vue";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();

const { active_profile } = storeToRefs(useRosterStore());
const {
  roster_config,
  active_roster_mats_owned,
  active_tradable_mats_owned,
  enabled_annotations,
} = storeToRefs(useRosterStore());
const histogram_result = computed(() => runtime.histogram.result);

// Rust returns these indexed by material row, so they are keyed on the way in.
const tier = computed(() => active_profile.value.tier);
const zeroes = (): number[] => ALL_LABELS[tier.value].map(() => 0);

// This is average mats cost (not gold)
const average_breakdown = computed(() =>
  by_label<number>(
    runtime.histogram.result?.avg_breakdown ?? zeroes(),
    tier.value,
  ),
);
// this is should always be treat tradable as bound (so it's actual gold spent)
const gold_breakdown = computed(() =>
  by_label<number>(
    runtime.histogram.result?.gold_breakdown_arr[0].map((x: number) =>
      x >= 0 ? 0 : -x,
    ) ?? zeroes(),
    tier.value,
  ),
);
const histogram_chances = computed(() =>
  by_label<number>(
    runtime.histogram.result?.chances_arr?.[
      active_profile.value.histogram_treatment_plan + 1
    ] ?? zeroes(),
    tier.value,
  ),
);

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
        has_upgrades_in_range(...range)
      );
    }),
);

const tickbox_tooltip = `Untick the box if you don't plan on buying that material from market.`;
// <span style="color:var(--text-muted)">(it also disable selling this mat)</span>`

const market_gold_text = "Avg gold spent buying from market";
const tradable_gold_text =
  "Avg gold spent buying minus gold from selling tradables";
const total_market_gold_text = "Average tradable gold spent:";
const total_market_gold_suffix = "(raw  +  buying needed mats & juice)";
const total_tradable_gold_text = "Avg sell value of leftover tradable mats:";
const total_tradable_gold_suffix = "(taxed)";
const all_bound_text = "Treat roster bound as tradable"; // i dont think i'll show this tho cos its kinda confusing
const selected_optimizer_treatement = ref(
  active_profile.value.optimizer_treatment_plan ==
    TreatmentPlan.TreatTradableAsBound
    ? market_gold_text
    : active_profile.value.optimizer_treatment_plan ==
        TreatmentPlan.TreatRosterAsBound
      ? tradable_gold_text
      : all_bound_text,
);

watchEffect(() => {
  if (selected_optimizer_treatement.value == market_gold_text) {
    active_profile.value.optimizer_treatment_plan =
      TreatmentPlan.TreatTradableAsBound;
  } else if (selected_optimizer_treatement.value == tradable_gold_text) {
    active_profile.value.optimizer_treatment_plan =
      TreatmentPlan.TreatRosterAsBound;
  } else if (selected_optimizer_treatement.value == all_bound_text) {
    active_profile.value.optimizer_treatment_plan =
      TreatmentPlan.TreatRosterAsTradable;
  }
});

const bound_chance_text =
  "Chance to succeed all upgrades within Char-Bound material";
const roster_chance_text =
  "Chance to succeed all upgrades within Roster-Bound material";
const tradable_chance_text =
  "Chance to succeed all upgrades within Tradable material";
// const chance_explainer_text = computed(() =>
//   active_profile.value.histogram_treatment_plan ==
//   TreatmentPlan.TreatTradableAsBound
//     ? tradable_chance_text
//     : active_profile.value.histogram_treatment_plan ==
//         TreatmentPlan.TreatRosterAsBound
//       ? roster_chance_text
//       : bound_chance_text,
// );

const selected_histogram_treatment = ref(
  active_profile.value.histogram_treatment_plan ==
    TreatmentPlan.TreatTradableAsBound
    ? tradable_chance_text
    : active_profile.value.histogram_treatment_plan ==
        TreatmentPlan.TreatRosterAsBound
      ? roster_chance_text
      : bound_chance_text,
);

const selected_histogram_color = ref(
  active_profile.value.histogram_treatment_plan ==
    TreatmentPlan.TreatRosterAsTradable
    ? "var(--bound)"
    : active_profile.value.histogram_treatment_plan ==
        TreatmentPlan.TreatRosterAsBound
      ? "var(--roster)"
      : "var(--tradable)",
); // initialize here otherwise it'll be null until we change it
function change_histogram_treatment(event) {
  let new_val = event.target.value;
  if (new_val === null) {
    return;
  }
  if (new_val == bound_chance_text) {
    active_profile.value.histogram_treatment_plan =
      TreatmentPlan.TreatRosterAsTradable;
  } else if (new_val == roster_chance_text) {
    active_profile.value.histogram_treatment_plan =
      TreatmentPlan.TreatRosterAsBound;
  } else if (new_val == tradable_chance_text) {
    active_profile.value.histogram_treatment_plan =
      TreatmentPlan.TreatTradableAsBound;
  }
  set_enabled();
  selected_histogram_color.value =
    active_profile.value.histogram_treatment_plan ==
    TreatmentPlan.TreatRosterAsTradable
      ? "var(--bound)"
      : active_profile.value.histogram_treatment_plan ==
          TreatmentPlan.TreatRosterAsBound
        ? "var(--roster)"
        : "var(--tradable)";
  runtime.histogram.throttled_start(WasmOp.Histogram, build_payload());
}

function set_enabled() {
  if (
    active_profile.value.histogram_treatment_plan ==
    TreatmentPlan.TreatRosterAsTradable
  ) {
    enabled_annotations.value[1] = true;
    enabled_annotations.value[2] = false;
    enabled_annotations.value[3] = false;
  } else if (
    active_profile.value.histogram_treatment_plan ==
    TreatmentPlan.TreatRosterAsBound
  ) {
    enabled_annotations.value[1] = false;
    enabled_annotations.value[2] = true;
    enabled_annotations.value[3] = false;
  } else if (
    active_profile.value.histogram_treatment_plan ==
    TreatmentPlan.TreatTradableAsBound
  ) {
    enabled_annotations.value[1] = false;
    enabled_annotations.value[2] = false;
    enabled_annotations.value[3] = true;
  }
}
onMounted(set_enabled); // overwrite the graph control options on load, which lowkey shouldn't even be there but whatever

const annotation_values = computed(() => {
  const bound = input_column_to_num(active_profile.value.bound_budgets);
  const roster = input_column_to_num(active_roster_mats_owned.value);
  const trade = input_column_to_num(active_tradable_mats_owned.value);
  const out: Record<string, number[]> = {};
  for (const label of ALL_LABELS[tier.value]) {
    out[label] = [
      average_breakdown.value[label],
      bound[label],
      roster[label] + bound[label],
      roster[label] + bound[label] + trade[label],
    ].filter((_, i) => enabled_annotations.value[i]);
  }
  return out;
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
    "minmax(190px, 250px) minmax(70px, 90px) minmax(110px, 120px) minmax(110px, 120px) 350px",
};

const is924Narrow = useMediaIsNarrow(924); // this turns out to be the width where the checkboxes overlap the labels
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
          <span class="w-25 text-left text-(--bound)">Bound Mats</span>
        </div>
        <select
          aria-label="Chance column"
          class="selector -mr-4! ml-4!"
          v-model="selected_histogram_treatment"
          :style="{
            color: selected_histogram_color,
          }"
          @change="change_histogram_treatment"
        >
          <option>{{ bound_chance_text }}</option>
          <option>{{ roster_chance_text }}</option>
          <option>{{ tradable_chance_text }}</option>
          <!-- <Select :options="items" optionLabel="name">
  <template #option="{ option }">
    <span style="font-family: 'Your Font', sans-serif;">{{ option.name }}</span>
  </template>
</Select> -->
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
            disabled:
              !active_profile.bound_budgets.enabled[label],
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
          />
          <!-- {{ console.log(averages) }} -->
          <MaterialCell
            :input_column="histogram_chances"
            :label="label"
            :input_color="selected_histogram_color"
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
            :data="histogram_result?.cum_percentiles?.[row] ?? null"
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
            :setter="
              (val) =>
                (active_profile.special_budget.values[
                  SPECIAL_LEAP_LABELS[active_profile.tier]
                ] = val)
            "
            :hide_tick="true"
            aria_name="Special leaps owned"
            :callback="() => start_all_workers()"
          ></MaterialCell>
          <span
            class="h-max text-xs leading-3 text-(--free-tap) underline hover:text-(--free-tap-bright)"
            @click="() => (show_special_guide = true)"
            >Should I use in T4 or convert?</span
          >
          <MaterialGraph
            :data="
              runtime.histogram.result.state_bundle?.latest_special_probs
                .concat(
                  new Array(
                    Math.max(
                      0,
                      runtime.histogram.result.state_bundle.upgrade_arr.filter(
                        (x) => x.is_normal_honing,
                      ).length -
                        runtime.histogram.result.state_bundle
                          ?.latest_special_probs.length,
                    ),
                  ).fill(0),
                )
                .slice(
                  0,
                  runtime.histogram.result.state_bundle.upgrade_arr.filter(
                    (x) => x.is_normal_honing,
                  ).length,
                )
                .map((x, index) => [index, x]) ?? null
            "
            :material-label="'Special'"
            :graph-color="'--free-tap'"
            :cumulative="roster_config.cumulative_graph"
            :tooltip-text-fn="special_hover_annotation"
            :max-yoverride="1"
            style="grid-column: span 3"
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
          {{
            metric_to_text(
              runtime.histogram.result?.metrics_arr[
                active_profile.optimizer_treatment_plan ===
                TreatmentPlan.TreatRosterAsBound
                  ? 0
                  : 1
              ],
            )
          }}
        </span>
        <span class="metric-result-suffix">
          {{ total_market_gold_suffix }}
        </span>
      </div>
    </div>
    <div
      v-if="
        active_profile.optimizer_treatment_plan ==
        TreatmentPlan.TreatRosterAsBound
      "
      class="metric-container"
    >
      <div class="metric-label text-(--text-muted)">
        {{ total_tradable_gold_text }}
      </div>
      <div class="metric-result-container">
        <span class="metric-result text-(--text-muted)">
          {{
            metric_to_text(
              runtime.histogram.result?.metrics_arr[0] -
                runtime.histogram.result?.metrics_arr[1],
            )
          }}
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
  flex-basis: 49.48%; /* (250+90+120+16)/962  */
  text-align: right;
  font-size: 1.25rem;
  line-height: 1.75rem;
}
/* aligning for the non-squished cases */
.metric-result-container {
  display: flex;
  min-width: calc(100% - 49.48%);
  flex-shrink: 1;
  flex-basis: calc(100% - 49.48%);
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
