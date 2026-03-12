<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { JUICE_LABELS, MATS_LABELS, PIECE_NAMES } from "@/Utils/Constants"
import { iconPath } from "@/Utils/Helpers"
import { StateBundle, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"

const profile_store = useProfilesStore()

const active_profile: CharProfile = profile_store.getActiveProfile()

const state_bundle: StateBundle = active_profile.state_bundle

const num_juice_avail: number = state_bundle.prep_output.juice_info.num_juice_avail
function goldBreakdownValue(offset: number) {
    const source = state_bundle?.average_breakdown ?? new Array(state_bundle.prep_output.juice_info.total_num_avail).fill(0)
    const value = Number(source[offset])
    return Number.isFinite(value) ? value : undefined
}
function breakdownText(value: number | undefined) {
    if (value === undefined || !Number.isFinite(value)) {
        return "N/A"
    }
    if (value <= 0) {
        return `Avg Eqv Cost ${Math.round(-value).toLocaleString("en-US")}g`
    }
    return `Avg Eqv Surplus ${Math.round(value).toLocaleString("en-US")}g`
}
function breakdownClass(value: number | undefined) {
    if (value === undefined || !Number.isFinite(value)) return "muted"
    return value > -0.5 ? "surplus" : "cost"
}
function metricToText(metric: number | null | undefined) {
    if (metric === null || metric === undefined || !Number.isFinite(metric)) return "N/A"
    return `${Math.round(-metric).toLocaleString("en-US")}g`
}
</script>
<template>
    <div class="hf-breakdown-grid">
        <div class="hf-breakdown-table">
            <div v-for="(label, index) in MATS_LABELS.slice(0, 7)" :key="`mats-breakdown-${label}`" class="hf-breakdown-row">
                <span class="hf-breakdown-label">{{ label }}</span>
                <span :class="['hf-breakdown-value', breakdownClass(goldBreakdownValue(index))]">
                    {{ breakdownText(goldBreakdownValue(index)) }}
                </span>
            </div>
        </div>

        <div class="hf-breakdown-table">
            <div v-for="(label, index) in JUICE_LABELS.map((pair) => pair[0])" :key="`weapon-breakdown-${label}`" class="hf-breakdown-row">
                <span class="hf-breakdown-label">{{ label }}</span>
                <span :class="['hf-breakdown-value', breakdownClass(goldBreakdownValue(7 + index))]">
                    {{ breakdownText(goldBreakdownValue(7 + index)) }}
                </span>
            </div>
        </div>

        <div class="hf-breakdown-table">
            <div v-for="(label, index) in JUICE_LABELS.map((pair) => pair[1])" :key="`armor-breakdown-${label}`" class="hf-breakdown-row">
                <span class="hf-breakdown-label">{{ label }}</span>
                <span :class="['hf-breakdown-value', breakdownClass(goldBreakdownValue(7 + num_juice_avail + index))]">
                    {{ breakdownText(goldBreakdownValue(7 + num_juice_avail + index)) }}
                </span>
            </div>
        </div>
    </div>

    <div class="hf-combined-cost">Combined: {{ metricToText(state_bundle.metric) }}</div>
</template>
