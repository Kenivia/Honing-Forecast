<script setup lang="ts">
import { ALL_LABELS, GRAPH_COLORS, T4_JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import GoldBreakdown from "./GoldBreakdown.vue"
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { iconPath } from "@/Utils/Helpers"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { create_input_column, HistogramOutputs, input_column_to_num, InputColumn, InputType } from "@/Utils/Interfaces"
import MaterialGraph from "./MaterialGraph.vue"
import { storeToRefs } from "pinia"
import { uesRosterStore } from "@/stores/RosterConfig"
import { computed, ref, Ref, toRef } from "vue"

const { active_profile } = storeToRefs(useProfilesStore())
const { roster_config } = storeToRefs(uesRosterStore())
const histogram_result = computed(() => active_profile.value.histogram_worker_bundle.result)
const averages: Ref<number[]> = toRef(() => histogram_result.value?.average ?? new Array(ALL_LABELS[active_profile.value.tier].length).fill(0))

const analysisTab = ref<"mats" | "juice">("mats")

const visibleRows = computed(() => {
    const matsIndices = MATS_LABELS.map((_, i) => i)

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
const computed_breakdown = computed(() => active_profile.value.optimizer_worker_bundle.result.average_breakdown.map((x) => (x == 0 ? x : -x)))
</script>

<template>
    <section class="hf-card hf-analysis-pane">
        <div class="hf-card-header">
            <div class="hf-card-title"><span class="hf-card-title-dot" />Costs</div>
            <div class="hf-analysis-tabs">
                <button :class="['hf-analysis-tab', { active: analysisTab === 'mats' }]" @click="analysisTab = 'mats'">Materials</button>
                <button :class="['hf-analysis-tab', { active: analysisTab === 'juice' }]" @click="analysisTab = 'juice'">Juice, Books & Scrolls</button>
            </div>
        </div>
        <div class="hf-card-body">
            <div class="hf-dist-desc">Distribution reflects free-tap and juice usage from your current optimizer output.</div>
            <div class="hf-dist-graphs">
                <div class="hf-table-title-row">
                    <span style="text-align: right; padding-right: 15px; color: var(--hf-graph-bound-color)">Char-Bound Mats</span>
                    <span style="color: var(--hf-graph-average-color)">Average Cost</span>
                    <span style="color: var(--hf-gold)">Avg gold spent </span>
                    <span style="text-align: center">Hover over the graph to see more!</span>
                    <!-- <span v-if="customLeftovers">Left</span> -->
                </div>
                <div
                    v-if="
                        ALL_LABELS[active_profile.tier].length == active_profile.bound_budgets[active_profile.tier].data.length &&
                        active_profile.optimizer_worker_bundle.result
                    "
                    v-for="{ label, row } in visibleRows"
                    :key="`graph-${label}`"
                    class="hf-mats-row"
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

                    <MaterialCell :input_column="averages" :row="row" :input_color="'--hf-graph-average-color'" />
                    <MaterialCell :input_column="computed_breakdown" :row="row" :input_color="'--hf-gold'" />
                    <MaterialGraph
                        :data="histogram_result?.cum_percentiles?.[row] ?? null"
                        :average="histogram_result?.average?.[row] ?? null"
                        :color-var="GRAPH_COLORS[row]"
                        :cumulative="roster_config.cumulative_graph"
                        :secondary-annotation="input_column_to_num(active_profile.bound_budgets[active_profile.tier])[row]"
                    />
                </div>
            </div>
        </div>
    </section>

    <GoldBreakdown />
</template>
<style>
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
    grid-template-columns: 250px 120px 120px minmax(0, 1fr);
    align-items: start;
    row-gap: 0;
}

.hf-table-title-row,
.hf-mats-row {
    display: grid;
    grid-column: 1 / -1; /* span all 3 columns */
    grid-template-columns: subgrid; /* inherit parent column definitions */
    align-items: center;
    border-bottom: 1px solid var(--separator-color);
}
</style>
