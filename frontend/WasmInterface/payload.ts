import { ALL_LABELS, JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import { WasmOp } from "./js_to_wasm"

import { CharProfile, TreatmentPlan, useProfilesStore } from "@/stores/CharacterProfile"
import { RosterConfig, uesRosterStore } from "@/stores/RosterConfig"

import { AdvProgress, keyed_to_array, MaterialInput, State, Upgrade, UpgradeInput, UpgradeStatus } from "@/Utils/Interfaces"

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
    const active_profile: CharProfile = useProfilesStore().getActiveProfile()
    const roster_config: RosterConfig = uesRosterStore().getRoster()

    if (wasm_op == WasmOp.EvaluateAverage || wasm_op == WasmOp.OptimizeAverage || wasm_op == WasmOp.Histogram) {
        return {
            material_info: ALL_LABELS.map((_, index) => [
                active_profile.bound_budgets.toNum()[index],
                roster_config.tradable_mats_owned.toNum()[index],
                active_profile.leftover_price.toNum()[index],
                roster_config.tradable_mats_price.toNum()[index],
                roster_config.mats_prices.toNum()[index],
            ]),
            upgrade_info: keyed_to_array(active_profile.KeyedUpgradeInput),
            special_budget: active_profile.special_budget,
            express_event: active_profile.express_event,
            tier: active_profile.tier,
            min_resolution: active_profile.min_resolution,
            num_threads: 1,
            metric_type: 1,
        }
    }
}
