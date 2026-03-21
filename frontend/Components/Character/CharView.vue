<script setup lang="ts">
import Instructions from "@/Components/Character/Instructions/Instructions.vue"
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue"
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue"
import { useProfilesStore } from "@/Stores/CharacterProfile"
import { useRosterStore } from "@/Stores/RosterConfig"
import { grids_to_keyed, StateBundle } from "@/Utils/Interfaces"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { build_material_info, build_payload } from "@/WasmInterface/payload"
import { storeToRefs } from "pinia"

import { onWatcherCleanup, watch } from "vue"
import { useRoute, useRouter } from "vue-router"

const route = useRoute()
const router = useRouter()

const profile_store = useProfilesStore()
const { active_profile } = storeToRefs(profile_store)
const { roster_config } = storeToRefs(useRosterStore())
// Route param → active character
watch(
    () => route.params.characterName as string,
    (name) => {
        const match = profile_store.profiles.findIndex((c) => c.char_name === name)
        if (match >= 0) {
            profile_store.active_profile_index = match
        } else {
            router.replace({ name: "char", params: { characterName: profile_store.profiles[0].char_name } })
        }
    },
    { immediate: true },
)

watch(
    [() => active_profile.value.adv_grid, () => active_profile.value.normal_grid, () => active_profile.value.tier],
    () => {
        console.log("keyed update")
        active_profile.value.keyed_upgrades = grids_to_keyed(
            active_profile.value.normal_grid,
            active_profile.value.adv_grid,
            active_profile.value.keyed_upgrades,
            active_profile.value.tier,
        )
    },
    { deep: true },
)

function start_eval_hist(result: StateBundle) {
    if (result === null) return
    active_profile.value.histogram_worker_bundle.throttled_start(WasmOp.Histogram, build_payload(WasmOp.Histogram, active_profile.value, roster_config.value))
    active_profile.value.evaluation_worker_bundle.throttled_start(
        WasmOp.EvaluateAverage,
        build_payload(WasmOp.EvaluateAverage, active_profile.value, roster_config.value),
    )
}

// Don't watch state changes betcause that's handled by start_eval_hist
watch(
    [
        () => active_profile.value.bound_budgets,
        () => active_profile.value.leftover_price,
        () => active_profile.value.tier,
        () => active_profile.value.express_event,
        () => active_profile.value.min_resolution,
        // () => roster_config.value.roster_mats_owned,
        // () => roster_config.value.tradable_mats_owned,
        // () => roster_config.value.mats_prices,
        () => active_profile.value.keyed_upgrades,
        () => active_profile.value.special_budget,
        () => active_profile.value.optimizer_treatment_plan,
    ],
    () => {
        onWatcherCleanup(() => {
            active_profile.value.optimizer_worker_bundle.cancel()
            active_profile.value.histogram_worker_bundle.cancel()
            active_profile.value.evaluation_worker_bundle.cancel()
        })
        console.log("payload update")
        let payload = build_payload(WasmOp.OptimizeAverage, active_profile.value, roster_config.value)

        if (active_profile.value.auto_start_optimizer) {
            active_profile.value.optimizer_worker_bundle.start(WasmOp.OptimizeAverage, payload, start_eval_hist)
        }
        payload.material_info = build_material_info(WasmOp.Histogram, active_profile.value, roster_config.value)
        active_profile.value.histogram_worker_bundle.throttled_start(WasmOp.Histogram, payload)

        payload.material_info = build_material_info(WasmOp.EvaluateAverage, active_profile.value, roster_config.value)
        active_profile.value.evaluation_worker_bundle.throttled_start(WasmOp.EvaluateAverage, payload)
    },
    { deep: true, immediate: true },
)
</script>

<template>
    <div class="hf-main-stage">
        <StatusInput />
        <MaterialDist />
        <Instructions />
    </div>
</template>
<style>
.hf-main-stage {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 8px;
    min-width: 0;
    width: 100%;
    justify-content: center;
    align-items: center;
}
</style>
