import { ALL_LABELS, JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import { WasmOp } from "./js_to_wasm"

import { CharProfile, TreatmentPlan, useProfilesStore } from "@/stores/CharacterProfile"
import { RosterConfig, uesRosterStore } from "@/stores/RosterConfig"

import { AdvProgress, keyed_to_array, MaterialInput, State, StateBundle, Upgrade, UpgradeInput, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { toRaw } from "vue"
import { mapToObject } from "@/Utils/Helpers"

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
function apply_treatement(treatment: TreatmentPlan, bound: number, roster: number, tradable: number): [number, number] {
    switch (treatment) {
        case TreatmentPlan.TreatTradableAsBound:
            return [bound + roster + tradable, 0]
        case TreatmentPlan.TreatRosterAsBound:
            return [bound + roster, tradable]
        case TreatmentPlan.TreatRosterAsTradable:
            return [bound, roster + tradable]
    }
}
export function buildPayload(wasm_op: WasmOp): EvalPayload | StateBundle {
    const { active_profile } = storeToRefs(useProfilesStore())
    const { roster_config } = storeToRefs(uesRosterStore())

    if (wasm_op == WasmOp.Parser) {
        const bound_budgets = active_profile.value.bound_budgets.toNum()
        const roster_mats_owned = roster_config.value.roster_mats_owned.toNum()
        const tradable_mats_owned = roster_config.value.tradable_mats_owned.toNum()
        const leftover_price = active_profile.value.leftover_price.toNum()
        const tradable_mats_price = roster_config.value.tradable_mats_price.toNum()
        const mats_prices = roster_config.value.mats_prices.toNum()
        return {
            material_info: ALL_LABELS.map((_, index) => [
                ...apply_treatement(active_profile.value.treatment_plan, bound_budgets[index], roster_mats_owned[index], tradable_mats_owned[index]),
                leftover_price[index],
                tradable_mats_price[index],
                mats_prices[index],
            ]),
            upgrade_info: keyed_to_array(active_profile.value.KeyedUpgradeInput),
            special_budget: active_profile.value.special_budget,
            express_event: active_profile.value.express_event,
            tier: active_profile.value.tier,
            min_resolution: active_profile.value.min_resolution,
            num_threads: 1,
            metric_type: 1,
        }
    } else {
        console.log(toRaw(active_profile.value.state_bundle))
        let out = toRaw(active_profile.value.state_bundle)
        for (let index = 0; index < out.prep_output.juice_info.all_juices.length; index++) {
            out.prep_output.juice_info.all_juices[index].data = mapToObject(out.prep_output.juice_info.all_juices[index].data)
        }
        return out
    }
}
