import { ALL_LABELS, BUNDLE_SIZE } from "@/Utils/Constants"
import { TreatmentPlan } from "@/Stores/CharacterProfile"
import { KeyedUpgrades, OneMaterial, OneUpgrade, Upgrade, WasmOp } from "@/Utils/Interfaces"
import { toRaw } from "vue"
import { useRosterStore } from "@/Stores/RosterConfig"
import { to_upgrade_key } from "@/Utils/KeyedUpgrades"
import { input_column_to_num } from "@/Utils/InputColumn"
import { storeToRefs } from "pinia"

// I don't think it's possible to directly export this struct from rust to javascript because of all the vectors,
// so it's copied & pasted here
export interface EvalPayload {
    material_info: OneMaterial[]
    upgrade_info: OneUpgrade[]
    special_budget: number
    special_state?: number[]
    tier: number
    express_event: boolean
    min_resolution: number
    num_threads: number
    metric_type: number
}

function keyed_to_array(keyed_upgrades: KeyedUpgrades, upgrade_arr: Upgrade[] | null, tier: number): OneUpgrade[] {
    return Object.entries(keyed_upgrades).map(([key, arr], index) => {
        arr[4] = null
        const out = structuredClone(toRaw(arr)) // cloning here so we don't write the state into keyed_upgrades
        let candidate = upgrade_arr?.[index]?.state ?? null
        if (candidate === null) {
            out[4] = []
        } else {
            let upgrade: Upgrade = upgrade_arr[index]
            if (to_upgrade_key(upgrade.piece_type, upgrade.upgrade_index, upgrade.is_normal_honing, tier) == key) {
                out[4] = candidate
            } else {
                out[4] = []
            }
        }
        return out
    })
}
export function apply_treatement(treatment: TreatmentPlan, bound: number, roster: number, tradable: number): [number, number] {
    switch (treatment) {
        case TreatmentPlan.TreatTradableAsBound:
            return [bound + roster + tradable, 0]
        case TreatmentPlan.TreatRosterAsBound:
            return [bound + roster, tradable]
        case TreatmentPlan.TreatRosterAsTradable:
            return [bound, roster + tradable]
        case TreatmentPlan.TreatAllAsTradable:
            return [0, bound + roster + tradable]
    }
}

export function build_material_info(wasm_op: WasmOp): OneMaterial[] {
    const { active_profile } = storeToRefs(useRosterStore())
    const { roster_config, active_roster_mats_owned, active_tradable_mats_owned } = storeToRefs(useRosterStore())

    const tier = active_profile.value.tier
    const bound_budgets = input_column_to_num(active_profile.value.bound_budgets[tier])
    const roster_mats_owned = input_column_to_num(active_roster_mats_owned.value[tier])
    const tradable_mats_owned = input_column_to_num(active_tradable_mats_owned.value[tier])

    const leftover_price = input_column_to_num(active_profile.value.leftover_price[tier])
    const actual_price = tier == 0 ? input_column_to_num(roster_config.value.mats_prices[tier]) : roster_config.value.effective_serca_price

    const tradable_mats_price = actual_price.map(
        (x: number, index: number) =>
            Math.max(Math.min(1, x), Math.floor(x * 0.95)) /
            (ALL_LABELS[active_profile.value.tier][index] == "Shards" ? roster_config.value.selected_shard_bag_size : BUNDLE_SIZE[index]),
    )
    const mats_prices = actual_price.map(
        (x: number, index: number) =>
            x / (ALL_LABELS[active_profile.value.tier][index] == "Shards" ? roster_config.value.selected_shard_bag_size : BUNDLE_SIZE[index]),
    )
    return ALL_LABELS[tier].map((_, index) => [
        ...apply_treatement(
            wasm_op == WasmOp.OptimizeAverage
                ? active_profile.value.optimizer_treatment_plan
                : wasm_op == WasmOp.Histogram
                  ? active_profile.value.histogram_treatment_plan
                  : index == 5
                    ? TreatmentPlan.TreatAllAsTradable // special case for gold (show all non-char bound gold cost)
                    : TreatmentPlan.TreatTradableAsBound, // EvalAverage
            bound_budgets[index],
            roster_mats_owned[index],
            tradable_mats_owned[index],
        ),
        leftover_price[index],
        tradable_mats_price[index],
        mats_prices[index],
    ])
}

export function build_payload(wasm_op: WasmOp): EvalPayload {
    const { active_profile } = storeToRefs(useRosterStore())
    const tier = active_profile.value.tier
    return {
        material_info: build_material_info(wasm_op),
        upgrade_info: keyed_to_array(active_profile.value.keyed_upgrades, active_profile.value.optimizer_worker_bundle.result?.upgrade_arr, tier),
        special_budget: input_column_to_num(active_profile.value.special_budget)[0],
        express_event: active_profile.value.express_event,
        tier,
        min_resolution: active_profile.value.min_resolution,
        num_threads: 1,
        metric_type: 1,
        special_state: toRaw(active_profile.value.optimizer_worker_bundle.result?.special_state),
    }
}
