<script setup lang="ts">
import { TreatmentPlan } from "@/Stores/CharacterProfile"
import { build_payload } from "@/WasmInterface/PayloadBuilder"
import { useRosterStore } from "@/Stores/RosterConfig"
import { storeToRefs } from "pinia"
import { ref, watchEffect } from "vue"
import { WasmOp } from "@/Utils/Interfaces"

const store = useRosterStore()
const { active_profile } = storeToRefs(store)

const { roster_config,} = storeToRefs(useRosterStore())
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
        <div class="hf-card-header" >
            <div class="hf-card-title">Controls</div>
        </div>
        <div class="hf-card-body hf-options-body" ">
            <label class="hf-inline-check">
                <input v-model="active_profile.express_event" type="checkbox" />
                <span>Express event (March)</span>
            </label>

            <!-- This is for producing payloads to feed into Rust -->
            <!-- <button class="hf-control-panel-btn" @click="copyPayload">Copy Payload</button> -->

            <label class="hf-inline-check">
                <input v-model="roster_config.cumulative_graph" type="checkbox" />
                <span>Cumulative graph</span>
            </label>

            <label class="hf-inline-check">
                <input v-model="treatment_tick" type="checkbox" />
                <span>Account for sell value of tradable mats (Recommended)</span>
            </label>

     
            <label class="hf-inline-check">
                <input v-model="active_profile.auto_start_optimizer" type="checkbox" />
                <span>Auto start optimizer</span>
            </label>
            <label class="hf-inline-check">
                <button class="hf-control-panel-btn" @click="resetActive">Reset this char</button>
            </label>
        </div>
    </section>
</template>
<style scoped>
.hf-control-panel-btn {
    color: var(--text-muted);
    margin-left: 20px;
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
    padding: 2px 0px;
    border-bottom: 1px solid var(--hf-border-subtle);
}
.hf-options-body {
    padding: 4px 0px;
}
</style>
