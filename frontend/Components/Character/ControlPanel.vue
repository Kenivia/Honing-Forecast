<script setup lang="ts">
import { TreatmentPlan, useProfilesStore } from "@/Stores/CharacterProfile"
import { build_payload } from "@/WasmInterface/payload"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { useRosterStore } from "@/Stores/RosterConfig"
import { storeToRefs } from "pinia"
import { ref, watchEffect } from "vue"

const store = useProfilesStore()
const { active_profile } = storeToRefs(store)

const { roster_config } = storeToRefs(useRosterStore())
function resetActive() {
    store.resetActiveProfile()
}

// This is useful for producing payloads to test the rust side
function copyPayload() {
    const payload = JSON.stringify(build_payload(WasmOp.Parser, active_profile.value, roster_config.value), null, 2)
    navigator.clipboard?.writeText(payload).catch(() => undefined)
}

const optimizer_worker = active_profile.value.optimizer_worker_bundle

// Currently TreatRosterAsTradable is not selectable
const treatment_tick = ref(active_profile.value.optimizer_treatment_plan == TreatmentPlan.TreatRosterAsBound)
watchEffect(() => {
    console.log("changed")
    if (treatment_tick.value) {
        active_profile.value.optimizer_treatment_plan = TreatmentPlan.TreatRosterAsBound
    } else {
        active_profile.value.optimizer_treatment_plan = TreatmentPlan.TreatTradableAsBound
    }
})
</script>
<template>
    <section class="hf-card hf-control-panel">
        <div class="hf-card-header">
            <div class="hf-card-title"><span class="hf-card-title-dot" />Controls</div>
        </div>
        <div class="hf-card-body hf-options-body">
            <button class="hf-control-panel-btn" @click="resetActive">Reset this char</button>
            <div style="font-size: x-small; color: var(--text-very-muted); text-wrap-mode: wrap">You may need to reload after</div>

            <!-- This is for producing paylodas to feed into Rust -->
            <!-- <button class="hf-control-panel-btn" @click="copyPayload">Copy Payload</button> -->

            <div class="hf-divider" />
            <label class="hf-inline-check">
                <input v-model="roster_config.cumulative_graph" type="checkbox" />
                <span>Cumulative graph</span>
            </label>
            <div class="hf-divider" />
            <label class="hf-inline-check">
                <input v-model="treatment_tick" type="checkbox" />
                <span>Optimizer account for sell value of tradable mats (Recommended)</span>
            </label>

            <div class="hf-divider" />
            <label class="hf-inline-check">
                <input v-model="active_profile.auto_start_optimizer" type="checkbox" />
                <span>Auto start optimizer</span>
            </label>
            <div class="hf-divider" />
            <div class="optimizer-progress">
                <span>Optimizer progress: {{ Math.max(optimizer_worker.est_progress_percentage, 0.01).toFixed(2) }}%</span>
                <div class="progress-bar">
                    <div class="progress-fill" :style="{ width: `${optimizer_worker.est_progress_percentage}%` }" />
                </div>
            </div>
        </div>
    </section>
</template>
<style scoped>
.hf-control-panel-btn {
    color: var(--text-muted);
}
.hf-control-panel {
    width: 200px;
    min-width: 0;
}
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

@media (max-width: 1000px) {
    .hf-control-panel {
        width: 100%;
    }
}
</style>
