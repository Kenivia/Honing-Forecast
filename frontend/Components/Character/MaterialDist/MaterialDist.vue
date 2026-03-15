<script setup lang="ts">
import { ALL_LABELS, GRAPH_COLORS, T4_JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import GoldBreakdown from "./GoldBreakdown.vue"
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { iconPath } from "@/Utils/Helpers"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { create_input_column, HistogramOutputs, InputColumn, InputType } from "@/Utils/Interfaces"
import MaterialGraph from "./MaterialGraph.vue"
import { storeToRefs } from "pinia"
import { uesRosterStore } from "@/stores/RosterConfig"
import { Ref, toRef } from "vue"

const { active_profile } = storeToRefs(useProfilesStore())
const { roster_config } = storeToRefs(uesRosterStore())
const histogram_result: Ref<HistogramOutputs | null> = toRef(() => active_profile.value.histogram_worker_bundle.result)
const averages: Ref<number[]> = toRef(() => histogram_result.value?.average ?? new Array(ALL_LABELS[active_profile.value.tier].length).fill(0))
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
                    <span style="text-align: right; padding-right: 15px">Char-Bound Mats</span>
                    <span>Average Cost</span>
                    <span style="text-align: center">Hover over the graph to see more!</span>
                    <!-- <span v-if="customLeftovers">Left</span> -->
                </div>
                <div v-for="(label, row) in ALL_LABELS[active_profile.tier]" :key="`graph-${label}`" class="hf-mats-row">
                    <MaterialCell
                        :input_column="active_profile.bound_budgets[active_profile.tier]"
                        :row="row"
                        :show_label="true"
                        :setter="
                            (val) => {
                                active_profile.bound_budgets[active_profile.tier].data[row] = val
                            }
                        "
                    />

                    <MaterialCell :input_column="averages" :row="row" :show_label="false" />
                    <MaterialGraph
                        :data="histogram_result?.cum_percentiles?.[row] ?? null"
                        :average="histogram_result?.average?.[row] ?? null"
                        :color-var="GRAPH_COLORS[row]"
                        :cumulative="roster_config.cumulative_graph"
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
    gap: 0; /* optional: spacing between cells */
}

.hf-table-title-row {
    display: contents;
}
</style>
