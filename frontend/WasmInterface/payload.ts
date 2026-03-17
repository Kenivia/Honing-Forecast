import { ALL_LABELS, BUNDLE_SIZE, T4_JUICE_LABELS, T4_MATS_LABELS } from "@/Utils/Constants"
import { WasmOp } from "./js_to_wasm"

import { CharProfile, TreatmentPlan, useProfilesStore } from "@/stores/CharacterProfile"
import { RosterConfig, useRosterStore } from "@/stores/RosterConfig"

import {
    AdvProgress,
    grids_to_keyed,
    input_column_to_num,
    keyed_to_array,
    MaterialInput,
    State,
    StateBundle,
    Upgrade,
    UpgradeInput,
    UpgradeStatus,
} from "@/Utils/Interfaces"
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
    const { roster_config } = storeToRefs(useRosterStore())

    const tier = active_profile.value.tier

    const bound_budgets = input_column_to_num(active_profile.value.bound_budgets[tier])
    const roster_mats_owned = input_column_to_num(roster_config.value.roster_mats_owned[tier])
    const tradable_mats_owned = input_column_to_num(roster_config.value.tradable_mats_owned[tier])
    const leftover_price = input_column_to_num(active_profile.value.leftover_price[tier])
    const tradable_mats_price = input_column_to_num(roster_config.value.mats_prices[tier]).map(
        (x: number, index: number) => Math.max(Math.min(1, x), Math.floor(x * 0.95)) / BUNDLE_SIZE[index],
    )
    const mats_prices = input_column_to_num(roster_config.value.mats_prices[tier]).map((x: number, index: number) => x / BUNDLE_SIZE[index])

    active_profile.value.keyed_upgrades = grids_to_keyed(active_profile.value.normal_grid, active_profile.value.adv_grid, active_profile.value.keyed_upgrades)

    // console.log(active_profile.value.keyed_upgrades)
    // console.log(keyed_to_array(active_profile.value.keyed_upgrades))
    return {
        material_info: ALL_LABELS[tier].map((_, index) => [
            ...apply_treatement(active_profile.value.treatment_plan, bound_budgets[index], roster_mats_owned[index], tradable_mats_owned[index]),
            leftover_price[index],
            tradable_mats_price[index],
            mats_prices[index],
        ]),
        upgrade_info: keyed_to_array(active_profile.value.keyed_upgrades, active_profile.value.optimizer_worker_bundle.result),
        special_budget: input_column_to_num(active_profile.value.special_budget)[0],
        express_event: active_profile.value.express_event,
        tier,
        min_resolution: active_profile.value.min_resolution,
        num_threads: 1,
        metric_type: 1,
    }

    //     else {
    //     // console.log(toRaw(active_profile.value.state_bundle))
    //     let out = toRaw(active_profile.value.state_bundle)
    //     for (let index = 0; index < out.prep_output.juice_info.all_juices.length; index++) {
    //         out.prep_output.juice_info.all_juices[index].data = mapToObject(out.prep_output.juice_info.all_juices[index].data)
    //     }
    //     return out
    // }
}
