<script setup lang="ts">
import {
  ALL_LABELS,
  GRAPH_COLORS,
  T4_MATS_LABELS,
  ANNOTATION_COLORS,
  ANNOTATION_POSITIONS,
  ANNOTATION_LABELS,
  SERCA_TO_T4_INDICES,
} from "@/Utils/Constants";
import { TreatmentPlan } from "@/Stores/CharacterProfile";
import { has_upgrades_in_range, metric_to_text } from "@/Utils/Helpers";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import MaterialGraph from "./MaterialGraph.vue";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { computed, ref, watchEffect } from "vue";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { input_column_to_num } from "@/Utils/InputColumn";
import { start_all_workers } from "../CharWorkerUtils";
import { RouterLink } from "vue-router";
import { WasmOp } from "@/WasmInterface/WasmWorker";
import { GridConfig } from "@/Utils/GridStyling";
import { useMediaIsNarrow } from "@/Utils/WindowSize";

const { active_profile } = storeToRefs(useRosterStore());
const {
  roster_config,
  active_roster_mats_owned,
  active_tradable_mats_owned,
  enabled_annotations,
} = storeToRefs(useRosterStore());
const histogram_result = computed(
  () => active_profile.value.histogram_worker_bundle.result,
);

// This is average mats cost (not gold)
const average_breakdown = computed(
  () =>
    active_profile.value.histogram_worker_bundle.result?.avg_breakdown ??
    new Array(ALL_LABELS[active_profile.value.tier].length).fill(0),
);
// this is should always be treat tradable as bound (so it's actual gold spent)
const gold_breakdown = computed(
  () =>
    active_profile.value.histogram_worker_bundle.result?.gold_breakdown_arr[0].map(
      (x: number) => Math.round(x >= 0 ? 0 : -x),
    ) ?? new Array(ALL_LABELS[active_profile.value.tier].length).fill(0),
);

const matsIndices = T4_MATS_LABELS.map((_, i) => i);
const visibleRows = computed(() => {
  return ALL_LABELS[active_profile.value.tier]
    .map((label, row) => ({ label, row })) // keep original index
    .filter(({ label, row }) => {
      return (
        matsIndices.includes(row) ||
        label === "Lava's Breath" ||
        label === "Glacier's Breath" ||
        (active_profile.value.tier === 0 &&
          ((has_upgrades_in_range(11, 14, true, false) &&
            label === "11-14 Weapon") ||
            (has_upgrades_in_range(15, 18, true, false) &&
              label === "15-18 Weapon") ||
            (has_upgrades_in_range(19, 20, true, false) &&
              label === "19-20 Weapon") ||
            (has_upgrades_in_range(11, 14, false, false) &&
              label === "11-14 Armor") ||
            (has_upgrades_in_range(15, 18, false, false) &&
              label === "15-18 Armor") ||
            (has_upgrades_in_range(19, 20, false, false) &&
              label === "19-20 Armor") ||
            (has_upgrades_in_range(1, 1, true, true) &&
              label === "Scroll 1 Weapon") ||
            (has_upgrades_in_range(2, 2, true, true) &&
              label === "Scroll 2 Weapon") ||
            (has_upgrades_in_range(3, 3, true, true) &&
              label === "Scroll 3 Weapon") ||
            (has_upgrades_in_range(4, 4, true, true) &&
              label === "Scroll 4 Weapon") ||
            (has_upgrades_in_range(1, 1, false, true) &&
              label === "Scroll 1 Armor") ||
            (has_upgrades_in_range(2, 2, false, true) &&
              label === "Scroll 2 Armor") ||
            (has_upgrades_in_range(3, 3, false, true) &&
              label === "Scroll 3 Armor") ||
            (has_upgrades_in_range(4, 4, false, true) &&
              label === "Scroll 4 Armor"))) ||
        average_breakdown.value[row] > 0.0 ||
        roster_config.value.show_all_rows
      );
    });
});

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
  "Chance to succeed all upgrades before running out of Char-Bound material of this type";
const roster_chance_text =
  "Chance to succeed all upgrades before running out of Roster-Bound material of this type";
const tradable_chance_text =
  "Chance to succeed all upgrades before running out of Tradable material of this type";
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
  selected_histogram_color.value =
    active_profile.value.histogram_treatment_plan ==
    TreatmentPlan.TreatRosterAsTradable
      ? "var(--bound)"
      : active_profile.value.histogram_treatment_plan ==
          TreatmentPlan.TreatRosterAsBound
        ? "var(--roster)"
        : "var(--tradable)";
  // console.log(new_val, active_profile.value.histogram_treatment_plan)
  active_profile.value.histogram_worker_bundle.throttled_start(
    WasmOp.Histogram,
    build_payload(),
  );
}

const annotation_values = computed(() => {
  let bound = input_column_to_num(
    active_profile.value.bound_budgets[active_profile.value.tier],
  );
  let roster = input_column_to_num(
    active_roster_mats_owned.value[active_profile.value.tier],
  );
  let trade = input_column_to_num(
    active_tradable_mats_owned.value[active_profile.value.tier],
  );
  return bound.map((_, i) =>
    [
      average_breakdown.value[i],
      bound[i],
      roster[i] + bound[i],
      roster[i] + bound[i] + trade[i],
    ].filter((_, i) => enabled_annotations.value[i]),
  );
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

  return `<b style="color: white;">${(cy * 100).toPrecision(place)}% </b> chance to use <br> &#9244;<b style="color: ${color};"> ${Math.ceil(x).toLocaleString("en-US")} </b> ${material_type} `;
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
    "minmax(180px, 250px) minmax(70px, 90px) minmax(110px, 120px) minmax(110px, 120px) 350px",
  grid_row_span: `span ${ALL_LABELS[1].length + 1}`,
};

const is924Narrow = useMediaIsNarrow(924); // this turns out to be the width where the checkboxes overlap the labels
</script>

<template>
  <section class="card-shell">
    <div class="card-header">
      <div class="card-title">Costs distribution</div>
    </div>
    <div
      class="card-body outer-grid pt-0! pb-2!"
      :style="{
        '--grid-cols': grid.grid_template_columns,
        gridRow: grid.grid_row_span,
      }"
    >
      <div class="mats-row h-fit! items-end! border-b-(--border-main)!">
        <div class="flex flex-row justify-between">
          <div
            class="question-mark mb-1"
            :style="{ textAlign: 'left', opacity: 1 }"
            v-tooltip="{
              value: tickbox_tooltip,
              escape: false,
            }"
          >
            <span class="self-center text-xs">?</span>
          </div>
          <span class="w-25 text-left text-(--bound)">Bound Mats</span>
        </div>
        <select
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
            class="w-0 basis-0 origin-right transform-[translateY(4px)] text-left text-xs text-(--text-very-muted)"
            >(tradable)</span
          >
        </div>

        <span class="text-(--text-muted)">Hover graph for details!</span>
        <!-- <span v-if="customLeftovers">Left</span> -->
      </div>
      <div
        v-if="
          ALL_LABELS[active_profile.tier].length ==
            active_profile.bound_budgets[active_profile.tier].data.length &&
          active_profile.histogram_worker_bundle.result &&
          active_profile.material_rerender_trigger
        "
        class="contents"
      >
        <div
          v-for="{ label, row } in visibleRows"
          :key="`graph-${label}`"
          class="mats-row"
          :class="{
            disabled:
              !active_profile.bound_budgets[active_profile.tier].enabled[row],
          }"
        >
          <MaterialCell
            :input_column="active_profile.bound_budgets[active_profile.tier]"
            :row="row"
            :label="label"
            :input_color="'--bound'"
            :setter="
              (val) => {
                active_profile.bound_budgets[active_profile.tier].data[row] =
                  val;
              }
            "
            :hide_tick="!matsIndices.includes(row)"
            :callback="() => start_all_workers()"
            :hide_label="is924Narrow && row < 7"
          />
          <!-- {{ console.log(averages) }} -->
          <MaterialCell
            :input_column="
              active_profile.histogram_worker_bundle.result.chances_arr[
                active_profile.histogram_treatment_plan + 1
              ]
            "
            :row="row"
            :input_color="selected_histogram_color"
            :is_percentage="true"
          />
          <MaterialCell
            :input_column="average_breakdown"
            :row="row"
            :input_color="'--average'"
          />
          <MaterialCell
            :input_column="gold_breakdown"
            :row="row"
            :input_color="'--gold'"
          />
          <MaterialGraph
            :data="histogram_result?.cum_percentiles?.[row] ?? null"
            :material-label="label"
            :graph-color="GRAPH_COLORS[active_profile.tier][row]"
            :cumulative="roster_config.cumulative_graph"
            :annotations="annotation_values[row]"
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
            :row="0"
            :setter="(val) => (active_profile.special_budget.data[0] = val)"
            :label="
              (active_profile.tier == 1 ? 'Serca ' : '') +
              active_profile.special_budget.keys[0]
            "
            :hide_tick="true"
            :callback="() => start_all_workers()"
          ></MaterialCell>
          <span
            class="text-xs text-(--free-tap) underline hover:text-(--free-tap-faded)"
            @click="() => (show_special_guide = true)"
            >Should I use in T4 or convert?</span
          >
          <MaterialGraph
            :data="
              active_profile.optimizer_worker_bundle.result?.latest_special_probs
                .concat(
                  new Array(
                    Math.max(
                      0,
                      active_profile.optimizer_worker_bundle.result.upgrade_arr.filter(
                        (x) => x.is_normal_honing,
                      ).length -
                        active_profile.optimizer_worker_bundle.result
                          ?.latest_special_probs.length,
                    ),
                  ).fill(0),
                )
                .slice(
                  0,
                  active_profile.optimizer_worker_bundle.result.upgrade_arr.filter(
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
            :upside_down_cumulative="true"
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
              active_profile.histogram_worker_bundle.result?.metrics_arr[0],
            ) ?? "No Result yet"
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
          TreatmentPlan.TreatRosterAsBound &&
        active_profile.auto_start_optimizer
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
              active_profile.histogram_worker_bundle.result?.metrics_arr[0] -
                active_profile.histogram_worker_bundle.result?.metrics_arr[1],
            ) ?? "No Result yet"
          }}
        </span>
        <span class="metric-result-suffix">
          {{ total_tradable_gold_suffix }}
        </span>
      </div>
    </div>

    <div class="mx-2 flex flex-row items-center gap-3">
      <span class="text-nowrap"
        >Optimizer progress:
        <span
          :style="{
            color:
              active_profile.optimizer_worker_bundle.est_progress_percentage ==
              100
                ? 'var(--gold)'
                : 'var(--warning-dark)',
          }"
        >
          {{
            active_profile.optimizer_worker_bundle.est_progress_percentage.toFixed(
              2,
            )
          }}%</span
        >
      </span>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{
            width: `${active_profile.optimizer_worker_bundle.est_progress_percentage}%`,
          }"
        />
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="show_special_guide"
        class="modal-overlay"
        @click="show_special_guide = false"
      >
        <div class="popup" @click.stop>
          <span style="font-size: 30px; color: var(--text-bright)">
            Short answer: Save Special Leaps and convert to Serca, unless you
            are tapping + 25
          </span>
          <span style="font-size: 16px; color: var(--text-muted)">
            If you're not + 20 yet, use it in T4.
          </span>
          <img src="/Special convert chart.png" alt="Special convert chart" />
        </div>
      </div>
    </Teleport>
  </section>
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
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: var(--bg-very-bright);
  border-radius: 4px;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  background: var(--gold);
  transition: width 0.1s ease;
}
/* .special-convert-guide {
  color: var(--free-tap);
  font-size: 12px;
  text-decoration-line: underline;
}

.special-convert-guide:hover {
  color: var(--free-tap-faded);
  font-size: 12px;
}

.optimizer-progress-label {
  display: flex;
  flex-direction: column;
  font-size: 16px;
  /* grid-column: 1 / span 2;
  text-align: right;
  padding: 6px;
  text-wrap-mode: nowrap;
}

.metric-label {
  grid-column: span 4;
  width: 100%;
  gap: 30px;
  color: var(--gold);
  font-size: 20px;
  text-align: right;
  padding-right: 8px;
  justify-content: center;
}

.smaller-label {
  font-size: 12px;
  color: var(--text-muted);
}

.metric-status {
  grid-column: 5 / span 2;
  width: 100%;
  gap: 30px;
  color: var(--gold);
  font-size: 30px;
  text-align: left;
  padding-right: 8px;
  justify-content: center;
  text-wrap-mode: nowrap;
}

.bound-header {
  color: var(--bound);
  text-align: right;
  padding-right: 8px;
}

.average-header {
  color: var(--average);
  text-align: center;
}

.gold-header {
  color: var(--gold);
  text-align: center;
}
.gold-header-suffix {
  color: var(--text-very-muted);
  font-size: 12px;
  min-width: 0;
  text-align: left;
  justify-self: right;
  margin-left: auto;
  position: absolute;
  transform: translateX(100%) translateY(4px);
  right: 22px;
}
.hover-hint {
  text-align: center;
  color: var(--text-muted);
  font-size: 11px;
}

.bound-select {
  min-width: 0;
}

.analysis-pane {
  width: min(100%, 992px);
  overflow-x: visible;
  overflow-y: visible;
}

.dist-scroll {
  width: 100%;
  overflow-x: auto;
  overflow-y: hidden;
  -webkit-overflow-scrolling: touch;
  /* scrollbar-gutter: stable; */
/* }

.dist-stack {
  display: flex;
  flex-direction: column;
  width: max-content;
  min-width: 100%;
}

.analysis-pane :deep(.card-header) {
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.analysis-pane :deep(.card-header > div) {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.analysis-tab {
  border: 1px solid var(--border-muted);
  border-radius: 999px;
  background: rgba(10, 13, 19, 0.48);
  color: var(--text-main);
  padding: 6px 12px;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  cursor: pointer;
}

.analysis-tab.active {
  background: rgba(212, 179, 90, 0.22);
  border-color: rgba(212, 179, 90, 0.6);
  color: var(--text-bright);
}
.dist-graphs {
  --dist-columns: 160px 90px 90px 120px 120px 350px;
  display: grid;
  grid-template-columns: var(--dist-columns);
  align-items: center;
  justify-content: start;
  row-gap: 0;
  min-width: max-content;
}

.table-title-row,
.mats-row {
  display: grid;
  grid-column: 1 / -1;
  grid-template-columns: var(--dist-columns);
  align-items: center;
  border-bottom: 1px solid var(--border-main);
  min-height: 0;
} */

/* @media (max-width: 900px) {
  .dist-graphs {
    --dist-columns: 100px 70px 70px 78px 78px 192px;
    min-width: max-content;
    width: auto;
  }

  .metric-label {
    grid-column: 1 / span 3;
    font-size: 16px;
    text-align: right;
    gap: 0;
  }

  .metric-status {
    grid-column: 4 / span 2;
    font-size: 22px;
    text-align: left;
    gap: 0;
  }

  .bound-select {
    width: 100%;
    font-size: 11px;
  }

  .bound-header,
  .average-header,
  .gold-header {
    font-size: 11px;
  }

  .average-header,
  .gold-header {
    text-align: center;
  }

  .table-title-row {
    font-size: 11px;
  }

  .hover-hint {
    font-size: 10px;
  }

  .mats-row :deep(.material-cell) {
    --cell-input-width: 64px;
    --cell-label-width: 88px;
    --cell-icon-size: 20px;
  }
} */
</style>
