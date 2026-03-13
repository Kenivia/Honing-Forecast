<script setup lang="ts">
import { ALL_LABELS, GRAPH_COLORS, JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import GoldBreakdown from "./GoldBreakdown.vue"
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { iconPath } from "@/Utils/Helpers"
import MaterialCell from "@/Components/MaterialCell.vue"
import { createInputColumn, InputType } from "@/Utils/Interfaces"
import MaterialGraph from "./MaterialGraph.vue"
import { storeToRefs } from "pinia"
import { uesRosterStore } from "@/stores/RosterConfig"


    const { active_profile } = storeToRefs(useProfilesStore())


const average_breakdown : number[] = active_profile.value?.state_bundle?.average_breakdown ?? new Array(ALL_LABELS.length).fill(0)
// const filtered_materials = unfiltered_materials.filter((x, index) => average_breakdown[index] > 0)

const histogram_result = active_profile.value.histogram_worker_bundle.result
// const keyed_average :Record<string,number> = Object.fromEntries(unfiltered_materials.map((x,index) => [x, flattened_bound_budgets[index]]));
</script>
<!-- function toInputValue(event: Event) {
    return (event.target as HTMLInputElement).value
}

function setRecordValue(record: Record<string, string>, key: string, event: Event) {
    record[key] = toInputValue(event)
} -->

<template>
    <section class="hf-card hf-analysis-pane">
        <div class="hf-card-header">
            <div class="hf-card-title"><span class="hf-card-title-dot" />Analysis</div>
        </div>
        <div class="hf-card-body">
            <div class="hf-dist-desc">Distribution reflects free-tap and juice usage from your current optimizer output.</div>
            <div class="hf-dist-graphs">
                <div class="hf-table-title-row"">
                    <span />
                    <span>Owned</span>
                    <span>Price</span>
                    <!-- <span v-if="customLeftovers">Left</span> -->
                </div>
                <div v-for="(label, index) in ALL_LABELS" :key="`graph-${label}`" class="hf-graph-row">
                    <div class="hf-graph-icon">
                        <img :src="iconPath(label)" :alt="label" />
                    </div>
                    <MaterialCell 
                    :input_columns="[active_profile.bound_budgets, average_breakdown ]"
                    :index="index"
                    ></MaterialCell>
                    <MaterialGraph
                        :data="histogram_result?.cum_percentiles?.[index] ?? null"
                        :average="histogram_result?.average?.[index] ?? null"
                        :secondary-annotation="histogram_result?.budgets?.[index] ?? null"
                        :color-var="GRAPH_COLORS[index]"
                        :cumulative="histogram_result"
                        :height="120"
                    />
                </div>
            </div>
        </div>
    </section>
   
    <GoldBreakdown />
</template>
