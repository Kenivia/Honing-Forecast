import { ALL_LABELS, BUNDLE_SIZE } from "@/Utils/Constants"
import { WasmOp } from "./js_to_wasm"
import { CharProfile, TreatmentPlan } from "@/Stores/CharacterProfile"
import { RosterConfig } from "@/Stores/RosterConfig"
import { input_column_to_num, KeyedUpgrades, OneMaterial, OneUpgrade, OneUpgradeKey, to_upgrade_key, Upgrade } from "@/Utils/Interfaces"
import { toRaw } from "vue"

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
        const out = toRaw(arr)
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
    }
}

export function build_material_info(wasm_op: WasmOp, active_profile: CharProfile, roster_config: RosterConfig): OneMaterial[] {
    const tier = active_profile.tier
    const bound_budgets = input_column_to_num(active_profile.bound_budgets[tier])
    const roster_mats_owned = input_column_to_num(roster_config.roster_mats_owned[tier])
    const tradable_mats_owned = input_column_to_num(roster_config.tradable_mats_owned[tier])

    const leftover_price = input_column_to_num(active_profile.leftover_price[tier])
    const actual_price = tier == 0 ? input_column_to_num(roster_config.mats_prices[tier]) : roster_config.effective_serca_price

    const tradable_mats_price = actual_price.map(
        (x: number, index: number) =>
            Math.max(Math.min(1, x), Math.floor(x * 0.95)) /
            (ALL_LABELS[active_profile.tier][index] == "Shards" ? roster_config.selected_shard_bag_size : BUNDLE_SIZE[index]),
    )
    const mats_prices = actual_price.map(
        (x: number, index: number) => x / (ALL_LABELS[active_profile.tier][index] == "Shards" ? roster_config.selected_shard_bag_size : BUNDLE_SIZE[index]),
    )
    return ALL_LABELS[tier].map((_, index) => [
        ...apply_treatement(
            wasm_op == WasmOp.OptimizeAverage
                ? active_profile.optimizer_treatment_plan
                : wasm_op == WasmOp.Histogram
                  ? active_profile.histogram_treatment_plan
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

export function build_payload(wasm_op: WasmOp, active_profile: CharProfile, roster_config: RosterConfig): EvalPayload {
    const tier = active_profile.tier
    return {
        material_info: build_material_info(wasm_op, active_profile, roster_config),
        upgrade_info: keyed_to_array(active_profile.keyed_upgrades, active_profile.optimizer_worker_bundle.result?.upgrade_arr, tier),
        special_budget: input_column_to_num(active_profile.special_budget)[0],
        express_event: active_profile.express_event,
        tier,
        min_resolution: active_profile.min_resolution,
        num_threads: 1,
        metric_type: 1,
        special_state: toRaw(active_profile.optimizer_worker_bundle.result?.special_state),
    }
}
