import { ALL_LABELS, JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import { WasmOp } from "./js_to_wasm"

import { CharProfile, TreatmentPlan, useProfilesStore } from "@/stores/CharacterProfile"
import { RosterConfig, uesRosterStore } from "@/stores/RosterConfig"

import { AdvProgress, keyed_to_array, MaterialInput, State, Upgrade, UpgradeInput, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"

function parseFloatZero(input: string) {
    const out = Number.parseFloat(input)
    return Number.isFinite(out) ? out : 0
}

// I don't think it's possible to directly export this struct from rust to javascript because of all the vectors,
// so it's copied & pasted here
export interface EvalPayload {
    material_info: MaterialInput
    upgrade_info: UpgradeInput
    special_budget: number
    special_state?: number[]
    tier: number
    express_event: boolean

    min_resolution: number

    num_threads: number

    metric_type: number
}
function apply_treatement(treatment: TreatmentPlan, bound: number, roster: number, tradable: number): { bound: number; tradable: number } {
    switch (treatment) {
        case TreatmentPlan.TreatTradableAsBound:
            return { bound: bound + roster + tradable, tradable: 0 }
        case TreatmentPlan.TreatRosterAsBound:
            return { bound: bound + roster, tradable }
        case TreatmentPlan.TreatRosterAsTradable:
            return { bound: bound, tradable: roster + tradable }
    }
}
export function buildPayload(wasm_op: WasmOp): EvalPayload {
    const { active_profile } = storeToRefs(useProfilesStore())
    const { roster_config } = storeToRefs(uesRosterStore())

    if (wasm_op == WasmOp.EvaluateAverage || wasm_op == WasmOp.OptimizeAverage || wasm_op == WasmOp.Histogram) {
        return {
            material_info: ALL_LABELS.map((_, index) => [
                active_profile.value.bound_budgets.toNum()[index],
                roster_config.value.tradable_mats_owned.toNum()[index],
                active_profile.value.leftover_price.toNum()[index],
                roster_config.value.tradable_mats_price.toNum()[index],
                roster_config.value.mats_prices.toNum()[index],
            ]),
            upgrade_info: keyed_to_array(active_profile.value.KeyedUpgradeInput),
            special_budget: active_profile.value.special_budget,
            express_event: active_profile.value.express_event,
            tier: active_profile.value.tier,
            min_resolution: active_profile.value.min_resolution,
            num_threads: 1,
            metric_type: 1,
        }
    }
}
