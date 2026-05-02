<script setup lang="ts">
import { TreatmentPlan } from "@/Stores/CharacterProfile"
import { build_payload } from "@/WasmInterface/PayloadBuilder"
import { useRosterStore } from "@/Stores/RosterConfig"
import { storeToRefs } from "pinia"
import { ref, watchEffect } from "vue"
import { WasmOp } from "@/Utils/Interfaces"

const store = useRosterStore()
const { active_profile } = storeToRefs(store)

const { roster_config, active_roster_mats_owned, active_tradable_mats_owned } = storeToRefs(useRosterStore())
function resetActive() {
    store.resetActiveProfile()
}

// This is useful for producing payloads to test the rust side
function copyPayload() {
    const payload = JSON.stringify(build_payload(WasmOp.Parser), null, 2)
    navigator.clipboard?.writeText(payload).catch(() => undefined)
}

// Currently TreatRosterAsTradable is not selectable
const treatment_tick = ref(active_profile.value.optimizer_treatment_plan == TreatmentPlan.TreatRosterAsBound)
watchEffect(() => {
    // console.log("changed")
    if (treatment_tick.value) {
        active_profile.value.optimizer_treatment_plan = TreatmentPlan.TreatRosterAsBound
    } else {
        active_profile.value.optimizer_treatment_plan = TreatmentPlan.TreatTradableAsBound
    }
})
</script>
<template>
    <section class="hf-control-panel">
        <div class="hf-card-header" style="margin-top: 100px">
            <div class="hf-card-title">Controls</div>
        </div>
        <div class="hf-card-body hf-options-body" ">
            <label class="hf-inline-check">
                <input v-model="active_profile.express_event" type="checkbox" />
                <span>Express event (March)</span>
            </label>

            <!-- This is for producing payloads to feed into Rust -->
            <!-- <button class="hf-control-panel-btn" @click="copyPayload">Copy Payload</button> -->

            <div class="hf-divider" />
            <label class="hf-inline-check">
                <input v-model="roster_config.cumulative_graph" type="checkbox" />
                <span>Cumulative graph</span>
            </label>
            <div class="hf-divider" />
            <label class="hf-inline-check">
                <input v-model="treatment_tick" type="checkbox" />
                <span>Account for sell value of tradable mats (Recommended)</span>
            </label>

            <div class="hf-divider" />
            <label class="hf-inline-check">
                <input v-model="active_profile.auto_start_optimizer" type="checkbox" />
                <span>Auto start optimizer</span>
            </label>
            <div class="hf-divider" />
            <button class="hf-control-panel-btn" @click="resetActive">Reset this char</button>
        </div>
    </section>
</template>
<style scoped>
.hf-control-panel-btn {
    color: var(--text-muted);
}
.hf-control-panel {
    min-width: 0;
    overflow-wrap: break-word;
    word-break: normal;
    align-items: center;
    font-size: 0.85rem;
    width: 100%;
}
.hf-inline-check {
    align-items: center;
    display: flex;
    flex-direction: row;
    padding-bottom: 6px;
}
.hf-options-body {
    padding: 4px 0px;
}
</style>
