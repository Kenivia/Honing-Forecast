<script setup lang="ts">
import {
    GRAPH_COLORS,
    T4_JUICE_LABELS,
    MATS_LABELS,
    TIER_LABELS,
    TIER_OPTIONS,
    ALL_LABELS,
    ADV_COLS,
    NUM_PIECES,
    NORMAL_COLS,
    PLUS_TIER_CONVERSION,
} from "@/Utils/Constants"
import GoldBreakdown from "./GoldBreakdown.vue"
import { CharProfile, create_default_char_profile, useProfilesStore } from "@/stores/CharacterProfile"
import { check_eligibility, iconPath, metricToText } from "@/Utils/Helpers"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { create_input_column, DEFAULT_ONE_UPGRADE, input_column_to_num, InputType, parse_input, UpgradeStatus } from "@/Utils/Interfaces"
import MaterialGraph from "./MaterialGraph.vue"
import { buildPayload } from "@/WasmInterface/payload"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { RosterConfig, uesRosterStore } from "@/stores/RosterConfig"
import { storeToRefs } from "pinia"
import { computed, watch } from "vue"

const store = useProfilesStore()
const { active_profile } = storeToRefs(store)

const { roster_config } = storeToRefs(uesRosterStore())
function resetActive() {
    store.resetActiveProfile()
}

// function resetOptimizerState() {
//     Object.entries(active_profile.value.keyed_upgrades).forEach(([_, one_upgrade]) =>
//         Object.assign(one_upgrade, [one_upgrade[0], one_upgrade[1], one_upgrade[2], ...DEFAULT_ONE_UPGRADE]),
//     )
// }
function copyPayload() {
    const payload = JSON.stringify(buildPayload(WasmOp.Parser), null, 2)
    navigator.clipboard?.writeText(payload).catch(() => undefined)
}

watch(
    () => active_profile.value.tier,

    (new_tier, old_tier) => {
        if (new_tier === null || old_tier === null || new_tier == old_tier) return
        if (ALL_LABELS.length != 2) {
            // This doesn't work for more tiers and should be updated when more tiers comes eventually
            throw new Error("conversion between more than 2 tiers not implemented yet")
        }

        active_profile.value.optimizer_worker_bundle?.cancel_and_clear_prev_result()
        active_profile.value.histogram_worker_bundle?.cancel_and_clear_prev_result()
        active_profile.value.evaluation_worker_bundle?.cancel_and_clear_prev_result()

        let num_array_old = input_column_to_num(active_profile.value.bound_budgets[old_tier])

        let multiplied_indices = [0, 1, 2, 4] // red, blue, leaps, fusion
        let multiplier = new_tier == 1 ? 0.2 : 5
        multiplied_indices.forEach(
            (index) =>
                (active_profile.value.bound_budgets[new_tier].data[index] = String(
                    parse_input(active_profile.value.bound_budgets[old_tier], index, String(num_array_old[index] * multiplier)),
                )),
        )
        // Special leaps also multiplied
        active_profile.value.special_budget.data[0] = String(
            parse_input(active_profile.value.special_budget, 0, String(input_column_to_num(active_profile.value.special_budget)[0] * multiplier)),
        )

        let stay_same_indices = [3, 5, 6, 7] // shards, gold, silver, red juice
        stay_same_indices.forEach(
            (index) => (active_profile.value.bound_budgets[new_tier].data[index] = active_profile.value.bound_budgets[old_tier].data[index]),
        )

        // special case for blue juice
        let new_num_juice_avail = (ALL_LABELS[new_tier].length - 7) / 2
        let new_index = 7 + new_num_juice_avail
        let old_num_juice_avail = (ALL_LABELS[old_tier].length - 7) / 2
        let old_index = 7 + old_num_juice_avail
        active_profile.value.bound_budgets[new_tier].data[new_index] = active_profile.value.bound_budgets[old_tier].data[old_index]

        // the rest have separate values between tiers

        for (let row = 0; row < NUM_PIECES; row++) {
            let highest_done = active_profile.value.normal_grid[row].findLastIndex((value) => value == UpgradeStatus.Done) + 1
            let highest_want = active_profile.value.normal_grid[row].findLastIndex((value) => value == UpgradeStatus.Want || value == UpgradeStatus.Done) + 1
            let converted_done = PLUS_TIER_CONVERSION[old_tier][String(highest_done)]
            let converted_want = PLUS_TIER_CONVERSION[old_tier]?.[String(highest_want)] ?? 25
            for (let col = 0; col < NORMAL_COLS; col++) {
                if (col < converted_done) {
                    active_profile.value.normal_grid[row][col] = UpgradeStatus.Done
                } else if (col < converted_want) {
                    active_profile.value.normal_grid[row][col] = UpgradeStatus.Want
                } else {
                    active_profile.value.normal_grid[row][col] = UpgradeStatus.NotYet
                }
            }
        }
    },
)
const optimizer_worker = active_profile.value.optimizer_worker_bundle

const optimizer_busy = computed(() => optimizer_worker.status === "busy")
const has_run_optimizer = computed(() => active_profile.value.has_run_optimizer)
const auto_start_optimizer = computed(() => active_profile.value.auto_start_optimizer)
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

                <div class="hf-divider" />
                <label class="hf-inline-check">
                    <input v-model="roster_config.cumulative_graph" type="checkbox" />
                    <span>Cumulative graph</span>
                </label>
                <div class="hf-divider" />

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
                @click="optimizer_worker.start(WasmOp.OptimizeAverage)"
            >
                {{ optimizer_busy ? "Cancel Optimize" : has_run_optimizer ? "Re-run Optimizer" : ">>> Optimize <<<" }}
            </button>

            <!-- <label class="hf-inline-check">
                <input v-model="store.profiles[active_profile_index].auto_start_optimizer" type="checkbox" />
                <span>Auto start optimizer</span>
            </label> -->

            <div class="hf-metric-card">
                <div class="hf-metric-label">Avg eqv gold cost</div>
                <div class="hf-metric-status">{{ metricToText(active_profile.optimizer_worker_bundle.result?.metric) ?? "No Result yet" }}</div>
            </div>

            <MaterialCell
                :input_column="active_profile.special_budget"
                :row="0"
                :setter="(val) => (active_profile.special_budget.data[0] = val)"
                :show_label="true"
            ></MaterialCell>

            <div v-if="optimizer_worker.status === 'error'" class="optimizer-error">Error: {{ optimizer_worker.error }}</div>

            <div class="optimizer-progress">
                <span>Optimizer progress: {{ Math.max(optimizer_worker.est_progress_percentage, 0.01).toFixed(2) }}%</span>
                <div class="progress-bar">
                    <div class="progress-fill" :style="{ width: `${optimizer_worker.est_progress_percentage}%` }" />
                </div>
            </div>
        </div>
    </div>
</template>
<style>
.progress-bar {
    width: 100%;
    height: 8px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    overflow: hidden;
}
.progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--hf-gold-dim), var(--hf-gold));
    transition: width 0.2s ease;
}
.optimizer-progress {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
}
</style>
