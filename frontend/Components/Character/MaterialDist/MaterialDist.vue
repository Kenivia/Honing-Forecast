<script setup lang="ts">
import { ALL_LABELS, GRAPH_COLORS, JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import GoldBreakdown from "./GoldBreakdown.vue"
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { iconPath } from "@/Utils/Helpers"
import MaterialCell from "@/Components/MaterialCell.vue"
import { createInputColumn, HistogramOutputs, InputColumn, InputType } from "@/Utils/Interfaces"
import MaterialGraph from "./MaterialGraph.vue"
import { storeToRefs } from "pinia"
import { uesRosterStore } from "@/stores/RosterConfig"
import { Ref, toRef } from "vue"

const { active_profile } = storeToRefs(useProfilesStore())
const averages: Ref<number[]> = toRef(() => histogram_result.value?.average ?? new Array(ALL_LABELS.length).fill(0))

const histogram_result: Ref<HistogramOutputs | null> = toRef(() => active_profile.value.histogram_worker_bundle.result)
</script>

<template>
    <section class="hf-card hf-analysis-pane">
        <div class="hf-card-header">
            <div class="hf-card-title"><span class="hf-card-title-dot" />Analysis</div>
        </div>
        <div class="hf-card-body">
            <div class="hf-dist-desc">Distribution reflects free-tap and juice usage from your current optimizer output.</div>
            <div class="hf-dist-graphs">
                <div class="hf-table-title-row">
                    <span style="text-align: right; padding-right: 15px">Character Bound Mats</span>
                    <span>Average Cost</span>
                    <span style="text-align: center">Hover over the graph to see more!</span>
                    <!-- <span v-if="customLeftovers">Left</span> -->
                </div>
                <div v-for="(label, index) in ALL_LABELS" :key="`graph-${label}`" class="hf-mats-row">
                    <MaterialCell
                        :input_column="active_profile.bound_budgets"
                        :row="index"
                        :show_label="true"
                        :setter="
                            (val) => {
                                active_profile.bound_budgets.data[index] = val
                            }
                        "
                    />

                    <MaterialCell :input_column="averages" :row="index" :show_label="false" />
                    <MaterialGraph
                        :data="histogram_result?.cum_percentiles?.[index] ?? null"
                        :average="histogram_result?.average?.[index] ?? null"
                        :color-var="GRAPH_COLORS[index]"
                    />
                </div>
            </div>
        </div>
    </section>

    <GoldBreakdown />
</template>
<style>
.hf-dist-graphs {
    display: grid;
    grid-template-columns: 250px 120px minmax(0, 1fr);
    align-items: center; /* optional: vertically center each cell */
    gap: 8px; /* optional: spacing between cells */
}

.hf-table-title-row {
    display: contents;
}
</style>
