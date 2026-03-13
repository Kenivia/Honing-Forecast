<script setup lang="ts">
import { GRAPH_COLORS, JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import GoldBreakdown from "./GoldBreakdown.vue"
import { CharProfile, createDefaultCharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { iconPath } from "@/Utils/Helpers"
import MaterialCell from "@/Components/MaterialCell.vue"
import { createInputColumn, DEFAULT_ONE_UPGRADE, InputType } from "@/Utils/Interfaces"
import MaterialGraph from "./MaterialGraph.vue"
import { buildPayload } from "@/WasmInterface/payload"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { RosterConfig, uesRosterStore } from "@/stores/RosterConfig"
import { storeToRefs } from "pinia"

const store = useProfilesStore()
const { active_profile, active_profile_index } = storeToRefs(store)

function resetActive() {
    store.profiles[active_profile_index.value] = createDefaultCharProfile()
}

// function resetOptimizerState() {
//     Object.entries(active_profile.value.KeyedUpgradeInput).forEach(([_, one_upgrade]) =>
//         Object.assign(one_upgrade, [one_upgrade[0], one_upgrade[1], one_upgrade[2], ...DEFAULT_ONE_UPGRADE]),
//     )
// }
function copyPayload() {
    const payload = JSON.stringify(buildPayload(WasmOp.EvaluateAverage), null, 2)
    navigator.clipboard?.writeText(payload).catch(() => undefined)
}

const optimizer_worker = active_profile.value.optimizer_worker_bundle

const optimizer_busy = optimizer_worker.status === "busy"
const has_run_optimizer = active_profile.value.has_run_optimizer
const auto_start_optimizer = active_profile.value.auto_start_optimizer
const optimizer_progress = optimizer_worker.est_progress_percentage
</script>
<template>
    <div class="hf-ops-row">
        <section class="hf-card">
            <div class="hf-card-header">
                <div class="hf-card-title"><span class="hf-card-title-dot" />Controls</div>
            </div>
            <div class="hf-card-body hf-options-body">
                <div class="hf-options-row">
                    <button class="hf-header-link-btn" @click="resetActive">Reset All</button>
                    <!-- <button class="hf-header-link-btn" @click="resetOptimizerState">Reset Optimizer</button> -->
                </div>
                <button class="hf-header-link-btn" @click="copyPayload">Copy Payload</button>

                <div class="hf-divider" />
                <label class="hf-inline-check">
                    <input v-model="active_profile.express_event" type="checkbox" />
                    <span>Express event</span>
                </label>
                <label class="hf-inline-check">
                    <input v-model="active_profile.cumulative_graph" type="checkbox" />
                    <span>Cumulative graph</span>
                </label>
                <!-- <label class="hf-inline-check">
                    <input v-model="allowManualState" type="checkbox" />
                    <span>Enable progress updates for better optimization</span>
                </label> -->
            </div>
        </section>
    </div>
    <div class="hf-card-header">
        <div class="hf-card-title"><span class="hf-card-title-dot" />Action Queue</div>
        <span class="hf-card-hint">Optimize, then follow next steps</span>
    </div>
    <div class="hf-card-body">
        <div class="optimizer-card">
            <button
                class="hf-optimize-btn"
                :style="{
                    background: optimizer_busy
                        ? 'var(--cancel-optimizer-button)'
                        : has_run_optimizer
                          ? 'linear-gradient(180deg, #60656f 0%, #4f545f 100%)'
                          : 'linear-gradient(180deg, #e6c86f 0%, #cfaf52 100%)',
                    color: optimizer_busy ? 'var(--text-muted)' : has_run_optimizer ? 'var(--hf-text-bright)' : '#1b1f25',
                }"
                @click="optimizer_worker.start(WasmOp.OptimizeAverage, buildPayload(WasmOp.OptimizeAverage))"
            >
                {{ optimizer_busy ? "Cancel Optimize" : has_run_optimizer ? "Re-run Optimizer" : ">>> Optimize <<<" }}
            </button>

            <label class="hf-inline-check">
                <input v-model="store.profiles[active_profile_index].auto_start_optimizer" type="checkbox" />
                <span>Auto start optimizer</span>
            </label>

            <div class="hf-metric-card">
                <div class="hf-metric-label">Avg eqv gold cost</div>
                <div class="hf-metric-status">{{ optimizer_worker.result?.metric ?? "No Result yet" }}</div>
            </div>

            <div v-if="optimizer_worker.status === 'error'" class="optimizer-error">Error: {{ optimizer_worker.error }}</div>

            <div v-if="optimizer_busy" class="optimizer-progress">
                <span>Optimizer progress: {{ Math.max(optimizer_progress, 0.01).toFixed(2) }}%</span>
                <div class="progress-bar">
                    <div class="progress-fill" :style="{ width: `${optimizer_progress}%` }" />
                </div>
            </div>
        </div>
    </div>
</template>
