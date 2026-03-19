<script setup lang="ts">
import { ALL_LABELS, GRAPH_COLORS, T4_JUICE_LABELS, T4_MATS_LABELS, ANNOTATION_COLORS, ANNOTATION_POSITIONS, ANNOTATION_LABELS } from "@/Utils/Constants"
import { CharProfile, TreatmentPlan, useProfilesStore } from "@/stores/CharacterProfile"
import { iconPath, metricToText } from "@/Utils/Helpers"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { create_input_column, HistogramOutputs, input_column_to_num, InputColumn, InputType } from "@/Utils/Interfaces"
import MaterialGraph from "./MaterialGraph.vue"
import { storeToRefs } from "pinia"
import { useRosterStore } from "@/stores/RosterConfig"
import { computed, nextTick, ref, Ref, toRef, watch, watchEffect } from "vue"

const { active_profile } = storeToRefs(useProfilesStore())
const { roster_config } = storeToRefs(useRosterStore())
const histogram_result = computed(() => active_profile.value.histogram_worker_bundle.result)
const average_breakdown = computed(
    () => active_profile.value.optimizer_worker_bundle.result?.average_breakdown ?? new Array(ALL_LABELS[active_profile.value.tier].length).fill(0),
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

// this is should always be treat tradable as bound
const gold_breakdown = computed(
    () =>
        active_profile.value.evaluation_worker_bundle.result?.gold_breakdown.map((x: number) => Math.ceil(x == 0 ? x : -x)) ??
        new Array(ALL_LABELS[active_profile.value.tier].length).fill(0),
)

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

const bound_chance_text = "Chance to succeed with bound"
const roster_chance_text = "Chance to succeed with bound + Roster bound"
const tradable_chance_text = "Chance to succeed with bound + Roster bound + Tradable"
const selected_histogram_treatment = ref(
    active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatTradableAsBound
        ? tradable_chance_text
        : active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatRosterAsBound
          ? roster_chance_text
          : bound_chance_text,
)

const selected_histogram_color = ref(null)
watchEffect(() => {
    if (selected_histogram_treatment.value == bound_chance_text) {
        active_profile.value.histogram_treatment_plan = TreatmentPlan.TreatRosterAsTradable
    } else if (selected_histogram_treatment.value == roster_chance_text) {
        active_profile.value.histogram_treatment_plan = TreatmentPlan.TreatRosterAsBound
    } else if (selected_histogram_treatment.value == tradable_chance_text) {
        active_profile.value.histogram_treatment_plan = TreatmentPlan.TreatTradableAsBound
    }
    selected_histogram_color.value =
        active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatRosterAsTradable
            ? "var(--hf-graph-bound-color)"
            : active_profile.value.histogram_treatment_plan == TreatmentPlan.TreatRosterAsBound
              ? "var(--hf-graph-roster-color)"
              : "var(--hf-graph-tradable-color)"
})
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

const re_render_trigger = ref(true)
const forceRerender = async () => {
    re_render_trigger.value = false
    await nextTick()
    re_render_trigger.value = true
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
            <div class="hf-dist-graphs">
                <div class="hf-table-title-row">
                    <span style="text-align: right; padding-right: 15px; color: var(--hf-graph-bound-color)">Char-Bound Mats</span>

                    <select
                        v-model="selected_histogram_treatment"
                        :style="{
                            color: selected_histogram_color,
                        }"
                    >
                        <option>{{ bound_chance_text }}</option>
                        <option>{{ roster_chance_text }}</option>
                        <option>{{ tradable_chance_text }}</option>
                    </select>
                    <span style="color: var(--hf-graph-average-color)">Average Cost</span>
                    <span style="color: var(--hf-gold)">Avg gold used</span>
                    <!-- <select v-model="selected_treatement" style="color: var(--hf-gold)">
                        <option>{{ market_gold_text }}</option>
                        <option>{{ tradable_gold_text }}</option>
                    </select> -->
                    <span style="text-align: center">Hover over the graph to see more!</span>
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

                    <div class="hf-mats-row">
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
                            :empty_message="'No free taps possible'"
                        />
                    </div>
                </div>
            </div>
            <div class="hf-dist-graphs">
                <div class="hf-mats-row">
                    <div class="hf-metric-label" style="grid-column: span 3">
                        {{ total_market_gold_text }}
                    </div>
                    <div class="hf-metric-status" style="grid-column: span 2">
                        {{ metricToText(active_profile.evaluation_worker_bundle.result?.metric) ?? "No Result yet" }}
                    </div>
                </div>
            </div>
        </div>
    </section>
</template>
<style>
.hf-metric-label {
    width: 100%;
    gap: 30px;
    color: var(--hf-gold);
    font-size: 20px;
    text-align: right;
    padding-right: 8px;
    justify-content: center;
}

.hf-metric-status {
    width: 100%;
    gap: 30px;
    color: var(--hf-gold);
    font-size: 30px;
    text-align: left;
    padding-right: 8px;
    justify-content: center;
}

.hf-analysis-pane {
    width: min(100%, 992px);
    overflow-x: auto;
    overflow-y: visible;
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
    display: grid;
    grid-template-columns: 250px 120px 120px 120px 320px;
    align-items: center;
    justify-content: start;
    row-gap: 0;
}

.hf-table-title-row,
.hf-mats-row {
    display: grid;
    grid-column: 1 / -1; /* span all 3 columns */
    grid-template-columns: subgrid; /* inherit parent column definitions */
    align-items: center;
    border-bottom: 1px solid var(--separator-color);
    min-height: 0px;
}

.hf-mats-row.disabled {
    opacity: 0.5;
}
</style>
