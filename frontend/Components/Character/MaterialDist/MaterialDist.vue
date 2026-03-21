<script setup lang="ts">
import { ALL_LABELS, GRAPH_COLORS, T4_MATS_LABELS, ANNOTATION_COLORS, ANNOTATION_POSITIONS, ANNOTATION_LABELS } from "@/Utils/Constants"
import { TreatmentPlan, useProfilesStore } from "@/Stores/CharacterProfile"
import { metricToText } from "@/Utils/Helpers"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { input_column_to_num } from "@/Utils/Interfaces"
import MaterialGraph from "./MaterialGraph.vue"
import { storeToRefs } from "pinia"
import { useRosterStore } from "@/Stores/RosterConfig"
import { computed, ref, watchEffect } from "vue"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { build_payload } from "@/WasmInterface/payload"

const { active_profile } = storeToRefs(useProfilesStore())
const { roster_config } = storeToRefs(useRosterStore())
const histogram_result = computed(() => active_profile.value.histogram_worker_bundle.result)

// This is average mats cost (not gold)
const average_breakdown = computed(
    () => active_profile.value.optimizer_worker_bundle.result?.average_breakdown ?? new Array(ALL_LABELS[active_profile.value.tier].length).fill(0),
)
// this is should always be treat tradable as bound (so it's actual gold spent)
const gold_breakdown = computed(
    () =>
        active_profile.value.evaluation_worker_bundle.result?.gold_breakdown.map((x: number) => Math.ceil(x == 0 ? x : -x)) ??
        new Array(ALL_LABELS[active_profile.value.tier].length).fill(0),
)

const analysisTab = ref<"mats" | "juice">("mats")
const visibleRows = computed(() => {
    const matsIndices = T4_MATS_LABELS.map((_, i) => i)
    return ALL_LABELS[active_profile.value.tier]
        .map((label, row) => ({ label, row })) // keep original index
        .filter(({ row }) => {
            if (analysisTab.value === "mats") {
                return matsIndices.includes(row)
            } else {
                return !matsIndices.includes(row)
            }
        })
})

const market_gold_text = "Avg gold spent buying from market"
const tradable_gold_text = "Avg gold spent buying minus gold from selling tradables"
const total_market_gold_text = "Avg total gold spent (raw + buying from market)"
const total_tradable_gold_text = "Avg total gold spent - the 'worth' of leftover tradable mats"
const all_bound_text = "Treat roster bound as tradable" // i dont think i'll show this tho cos its kinda confusing
const selected_optimizer_treatement = ref(
    active_profile.value.optimizer_treatment_plan == TreatmentPlan.TreatTradableAsBound
        ? market_gold_text
        : active_profile.value.optimizer_treatment_plan == TreatmentPlan.TreatRosterAsBound
          ? tradable_gold_text
          : all_bound_text,
)

watchEffect(() => {
    if (selected_optimizer_treatement.value == market_gold_text) {
        active_profile.value.optimizer_treatment_plan = TreatmentPlan.TreatTradableAsBound
    } else if (selected_optimizer_treatement.value == tradable_gold_text) {
        active_profile.value.optimizer_treatment_plan = TreatmentPlan.TreatRosterAsBound
    } else if (selected_optimizer_treatement.value == all_bound_text) {
        active_profile.value.optimizer_treatment_plan = TreatmentPlan.TreatRosterAsTradable
    }
})

const bound_chance_text = "Bound Chance"
const roster_chance_text = "Bound + Roster Chance"
const tradable_chance_text = "Bound + Roster + Tradable"
const selected_histogram_treatment = ref(
    active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatTradableAsBound
        ? tradable_chance_text
        : active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatRosterAsBound
          ? roster_chance_text
          : bound_chance_text,
)

const selected_histogram_color = ref(
    active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatRosterAsTradable
        ? "var(--hf-graph-bound-color)"
        : active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatRosterAsBound
          ? "var(--hf-graph-roster-color)"
          : "var(--hf-graph-tradable-color)",
) // initialize here otherwise it'll be null until we change it
function change_histogram_treatment(event) {
    let new_val = event.target.value
    if (new_val === null) {
        return
    }
    if (new_val == bound_chance_text) {
        active_profile.value.histogram_treatment_plan = TreatmentPlan.TreatRosterAsTradable
    } else if (new_val == roster_chance_text) {
        active_profile.value.histogram_treatment_plan = TreatmentPlan.TreatRosterAsBound
    } else if (new_val == tradable_chance_text) {
        active_profile.value.histogram_treatment_plan = TreatmentPlan.TreatTradableAsBound
    }
    selected_histogram_color.value =
        active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatRosterAsTradable
            ? "var(--hf-graph-bound-color)"
            : active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatRosterAsBound
              ? "var(--hf-graph-roster-color)"
              : "var(--hf-graph-tradable-color)"
    // console.log(new_val, active_profile.value.histogram_treatment_plan)
    active_profile.value.histogram_worker_bundle.throttled_start(WasmOp.Histogram, build_payload(WasmOp.Histogram, active_profile.value, roster_config.value))
}

const enabled_annotations = ref([true, false, false, false])
const annotation_values = computed(() => {
    let bound = input_column_to_num(active_profile.value.bound_budgets[active_profile.value.tier])
    let roster = input_column_to_num(roster_config.value.roster_mats_owned[active_profile.value.tier])
    let trade = input_column_to_num(roster_config.value.tradable_mats_owned[active_profile.value.tier])
    return bound.map((_, i) =>
        [average_breakdown.value[i], bound[i], roster[i] + bound[i], roster[i] + bound[i] + trade[i]].filter((_, i) => enabled_annotations.value[i]),
    )
})

function hover_annotation(x, _y, cy, material_type, color): string {
    let place = Math.min(10, Math.max(Math.ceil(cy < 0.5 ? Math.min(3, Math.abs(Math.log10(cy))) : Math.abs(Math.log10(1 - cy))), 3))
    return `<b style="color: white;">${(cy * 100).toPrecision(place)}% </b> chance to use <br> &#8804;<b style="color: ${color};"> ${Math.ceil(x).toLocaleString("en-US")} </b> ${material_type} `
}
function special_hover_annotation(x, _y, cy, material_type, color): string {
    let place = Math.min(10, Math.max(Math.ceil(cy < 0.5 ? Math.min(3, Math.abs(Math.log10(cy))) : Math.abs(Math.log10(1 - cy))), 3))
    return `<b style="color: white;">${(cy * 100).toPrecision(place)}% </b> chance to free tap <br> at least <b style="color: ${color};"> ${x + 1} </b> piece`
}
</script>

<template>
    <section class="hf-card hf-analysis-pane">
        <div class="hf-card-header">
            <div class="hf-card-title"><span class="hf-card-title-dot" />Costs</div>
            <div>
                <button :class="['hf-analysis-tab', { active: analysisTab === 'mats' }]" @click="analysisTab = 'mats'">Materials</button>
                <button :class="['hf-analysis-tab', { active: analysisTab === 'juice' }]" @click="analysisTab = 'juice'">Juice, Books & Scrolls</button>
            </div>
            <div>
                <button
                    v-for="(label, index) in ANNOTATION_LABELS"
                    :class="[`hf-graph-tab-${label.replace('+', '').toLowerCase()}`, { active: enabled_annotations[index] }]"
                    @click="enabled_annotations[index] = !enabled_annotations[index]"
                >
                    {{ label }}
                </button>
            </div>
        </div>
        <div class="hf-card-body">
            <div class="hf-dist-scroll">
                <div class="hf-dist-stack">
                    <div class="hf-dist-graphs">
                        <div class="hf-table-title-row">
                            <span class="hf-bound-header">Bound Mats</span>

                            <select
                                class="hf-bound-select"
                                v-model="selected_histogram_treatment"
                                :style="{
                                    color: selected_histogram_color,
                                }"
                                @change="change_histogram_treatment"
                            >
                                <option>{{ bound_chance_text }}</option>
                                <option>{{ roster_chance_text }}</option>
                                <option>{{ tradable_chance_text }}</option>
                            </select>
                            <span class="hf-average-header">Average</span>
                            <span class="hf-gold-header">Gold Used</span>
                            <span class="hf-hover-hint">Hover graph for details</span>
                            <!-- <span v-if="customLeftovers">Left</span> -->
                        </div>
                        <div
                            v-if="
                                ALL_LABELS[active_profile.tier].length == active_profile.bound_budgets[active_profile.tier].data.length &&
                                active_profile.optimizer_worker_bundle.result &&
                                active_profile.histogram_worker_bundle.result
                            "
                            style="display: contents"
                        >
                            <div
                                v-for="{ label, row } in visibleRows"
                                :key="`graph-${label}`"
                                class="hf-mats-row"
                                :class="{ disabled: !active_profile.bound_budgets[active_profile.tier].enabled[row] }"
                            >
                                <MaterialCell
                                    :input_column="active_profile.bound_budgets[active_profile.tier]"
                                    :row="row"
                                    :label="label"
                                    :input_color="'--hf-graph-bound-color'"
                                    :setter="
                                        (val) => {
                                            active_profile.bound_budgets[active_profile.tier].data[row] = val
                                        }
                                    "
                                    :hide_tick="analysisTab == 'juice'"
                                />
                                <!-- {{ console.log(averages) }} -->
                                <MaterialCell
                                    :input_column="active_profile.histogram_worker_bundle.result.bound_chance"
                                    :row="row"
                                    :input_color="selected_histogram_color"
                                    :is_percentage="true"
                                />
                                <MaterialCell :input_column="average_breakdown" :row="row" :input_color="'--hf-graph-average-color'" />
                                <MaterialCell :input_column="gold_breakdown" :row="row" :input_color="'--hf-gold'" />
                                <MaterialGraph
                                    :data="histogram_result?.cum_percentiles?.[row] ?? null"
                                    :material-label="label"
                                    :graph-color="GRAPH_COLORS[row]"
                                    :cumulative="roster_config.cumulative_graph"
                                    :annotations="annotation_values[row]"
                                    :annotationColors="ANNOTATION_COLORS.filter((_, i) => enabled_annotations[i])"
                                    :annotation-positions="ANNOTATION_POSITIONS.filter((_, i) => enabled_annotations[i])"
                                    :annotationLabels="ANNOTATION_LABELS.filter((_, i) => enabled_annotations[i])"
                                    :tooltip-text-fn="hover_annotation"
                                />
                            </div>

                            <!-- Special Re-render is trigger by the confirm button in instruction row because otherwise it wouldn't update -->
                            <div v-if="active_profile.special_re_render_trigger" class="hf-mats-row">
                                <MaterialCell
                                    :input_column="active_profile.special_budget"
                                    :row="0"
                                    :setter="(val) => (active_profile.special_budget.data[0] = val)"
                                    :label="(active_profile.tier == 1 ? 'Serca ' : '') + active_profile.special_budget.keys[0]"
                                    v-if="active_profile.special_re_render_trigger"
                                    :hide_tick="true"
                                ></MaterialCell>
                                <!-- {{ console.log(active_profile.optimizer_worker_bundle.result?.latest_special_probs) }} -->
                                <MaterialGraph
                                    :data="
                                        active_profile.optimizer_worker_bundle.result?.latest_special_probs
                                            .concat(
                                                new Array(
                                                    Math.max(
                                                        0,
                                                        active_profile.optimizer_worker_bundle.result.upgrade_arr.filter((x) => x.is_normal_honing).length -
                                                            active_profile.optimizer_worker_bundle.result?.latest_special_probs.length,
                                                    ),
                                                ).fill(0),
                                            )
                                            .slice(0, active_profile.optimizer_worker_bundle.result.upgrade_arr.filter((x) => x.is_normal_honing).length)
                                            .map((x, index) => [index, x]) ?? null
                                    "
                                    :material-label="'Special'"
                                    :graph-color="'--hf-free-tap'"
                                    :cumulative="roster_config.cumulative_graph"
                                    :tooltip-text-fn="special_hover_annotation"
                                    :max-yoverride="1"
                                    style="grid-column: span 4"
                                    :empty_message="'No normal honing available'"
                                />
                            </div>
                        </div>
                    </div>
                    <div class="hf-dist-graphs">
                        <div class="hf-mats-row">
                            <div class="hf-metric-label">
                                {{ total_market_gold_text }}
                            </div>
                            <div class="hf-metric-status">
                                {{ metricToText(active_profile.evaluation_worker_bundle.result?.metric) ?? "No Result yet" }}
                            </div>
                        </div>
                        <div class="hf-mats-row">
                            <span class="optimizer-progress"
                                >Optimizer progress: {{ Math.max(active_profile.optimizer_worker_bundle.est_progress_percentage, 0.01).toFixed(2) }}%
                            </span>
                            <div class="progress-bar">
                                <div class="progress-fill" :style="{ width: `${active_profile.optimizer_worker_bundle.est_progress_percentage}%` }" />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </section>
</template>
<style scoped>
.progress-bar {
    width: 100%;
    height: 8px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    overflow: hidden;
    grid-column: 2 / span 4;
}
.progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--hf-gold-dim), var(--hf-gold));
    transition: width 0.1s ease;
}
.optimizer-progress {
    display: flex;
    flex-direction: column;

    font-size: 12px;
    grid-column: 1 / span 1;
    text-align: right;
    padding: 6px;
}
.hf-metric-label {
    grid-column: 1 / span 3;
    width: 100%;
    gap: 30px;
    color: var(--hf-gold);
    font-size: 20px;
    text-align: right;
    padding-right: 8px;
    justify-content: center;
}

.hf-metric-status {
    grid-column: 4 / span 2;
    width: 100%;
    gap: 30px;
    color: var(--hf-gold);
    font-size: 30px;
    text-align: left;
    padding-right: 8px;
    justify-content: center;
}

.hf-bound-header {
    color: var(--hf-graph-bound-color);
    text-align: right;
    padding-right: 8px;
}

.hf-average-header {
    color: var(--hf-graph-average-color);
    text-align: center;
}

.hf-gold-header {
    color: var(--hf-gold);
    text-align: center;
}

.hf-hover-hint {
    text-align: center;
    color: var(--hf-text-muted);
    font-size: 11px;
}

.hf-bound-select {
    min-width: 0;
}

.hf-analysis-pane {
    width: min(100%, 992px);
    overflow-x: visible;
    overflow-y: visible;
}

.hf-dist-scroll {
    width: 100%;
    /* overflow-x: auto; */
    overflow-y: visible;
    -webkit-overflow-scrolling: touch;
}

.hf-dist-stack {
    display: flex;
    flex-direction: column;
    width: max-content;
    min-width: 100%;
}

.hf-analysis-pane :deep(.hf-card-header) {
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
}

.hf-analysis-pane :deep(.hf-card-header > div) {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
}

.hf-graph-tab-avg,
.hf-graph-tab-bound,
.hf-graph-tab-roster-bound,
.hf-graph-tab-tradable {
    border: 1px solid var(--hf-border-subtle);
    border-radius: 999px;
    background: rgba(10, 13, 19, 0.48);
    color: var(--hf-text-main);
    padding: 6px 12px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
}
.hf-graph-tab-avg.active {
    background: var(--hf-graph-average-color);
    border-color: var(--separator-color);
    color: var(--hf-bg-deep);
}

.hf-graph-tab-bound.active {
    background: var(--hf-graph-bound-color);
    border-color: var(--separator-color);
    color: var(--hf-bg-deep);
}

.hf-graph-tab-roster-bound.active {
    background: var(--hf-graph-roster-color);
    border-color: var(--separator-color);
    color: var(--hf-bg-deep);
}

.hf-graph-tab-tradable.active {
    background: var(--hf-graph-tradable-color);
    border-color: var(--separator-color);
    color: var(--hf-bg-deep);
}

.hf-analysis-tab {
    border: 1px solid var(--hf-border-subtle);
    border-radius: 999px;
    background: rgba(10, 13, 19, 0.48);
    color: var(--hf-text-main);
    padding: 6px 12px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
}

.hf-analysis-tab.active {
    background: rgba(212, 179, 90, 0.22);
    border-color: rgba(212, 179, 90, 0.6);
    color: var(--hf-text-bright);
}
.hf-dist-graphs {
    --hf-dist-columns: 250px 120px 120px 120px 320px;
    display: grid;
    grid-template-columns: var(--hf-dist-columns);
    align-items: center;
    justify-content: start;
    row-gap: 0;
    min-width: max-content;
}

.hf-table-title-row,
.hf-mats-row {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: var(--hf-dist-columns);
    align-items: center;
    border-bottom: 1px solid var(--separator-color);
    min-height: 0;
}

.hf-mats-row.disabled {
    opacity: 0.5;
}

@media (max-width: 900px) {
    .hf-dist-graphs {
        --hf-dist-columns: 170px 112px 78px 78px 150px;
        min-width: max-content;
        width: auto;
    }

    .hf-metric-label {
        grid-column: 1 / span 3;
        font-size: 16px;
        text-align: right;
        gap: 0;
    }

    .hf-metric-status {
        grid-column: 4 / span 2;
        font-size: 22px;
        text-align: left;
        gap: 0;
    }

    .hf-bound-select {
        width: 100%;
        font-size: 11px;
    }

    .hf-bound-header,
    .hf-average-header,
    .hf-gold-header {
        font-size: 11px;
    }

    .hf-average-header,
    .hf-gold-header {
        text-align: center;
    }

    .hf-analysis-tab,
    .hf-graph-tab-avg,
    .hf-graph-tab-bound,
    .hf-graph-tab-roster-bound,
    .hf-graph-tab-tradable {
        padding: 5px 10px;
        font-size: 11px;
    }

    .hf-table-title-row {
        font-size: 11px;
    }

    .hf-hover-hint {
        font-size: 10px;
    }

    .hf-mats-row :deep(.hf-material-cell) {
        --hf-cell-input-width: 64px;
        --hf-cell-label-width: 88px;
        --hf-cell-icon-size: 20px;
    }
}
</style>
