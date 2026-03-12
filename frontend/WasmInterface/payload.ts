import { JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import { WasmOp } from "./js_to_wasm"

import { CharProfile, TreatmentPlan, useProfilesStore } from "@/stores/CharacterProfile"
import { RosterConfig, uesRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES } from "@/Utils/Constants"
import { iconPath } from "@/Utils/Helpers"
import { AdvProgress, State, Upgrade, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { computed } from "vue"
import InstructionRow from "./InstructionRow.vue"

const active_profile: CharProfile = useProfilesStore().activeProfile()

const roster_config: RosterConfig = uesRosterStore().getRoster()

const optimizer_worker = active_profile.optimizer_worker_bundle
const optimizer_busy = optimizer_worker.status === "running" || optimizer_worker.status === "pending"
const has_run_optimizer = active_profile.has_run_optimizer
const auto_start_optimizer = active_profile.auto_start_optimizer
const optimizer_progress = optimizer_worker.est_progress_percentage

function parseFloatZero(input: string) {
    const out = Number.parseFloat(input)
    return Number.isFinite(out) ? out : 0
}

// I don't think it's possible to directly export this struct from rust to javascript because of all the vectors,
// so it's copied & pasted here
export interface EvalPayload {
    normal_hone_ticks: Boolean[][]
    adv_hone_ticks: Boolean[][]
    express_event: Boolean

    inp_bound_mats: number[]
    inp_trade_mats: number[]

    inp_market_mats_price: number[]
    inp_trade_mats_price: number[]
    inp_left_mats_price: number[]

    inp_bound_juice: number[][]
    inp_trade_juice: number[][]

    inp_juice_market_price: number[][]
    inp_juice_trade_price: number[][]
    inp_juice_left_price: number[][]

    normal_progress_grid?: number[][]
    normal_state_grid?: State[][][]
    special_state?: number[]
    normal_unlock_grid?: Boolean[][]
    succeeded_grid?: Boolean[][]
    adv_progress_grid?: AdvProgress[][]
    tier: number
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
    let mats = active_profile.bound_mats_owned.keys.map((key) =>
        apply_treatement(
            active_profile.treatment_plan,
            active_profile.bound_mats_owned[key],
            roster_config.roster_mats_owned[key],
            roster_config.tradable_mats_owned[key],
        ),
    )
    if (wasm_op == WasmOp.EvaluateAverage || wasm_op == WasmOp.OptimizeAverage || wasm_op == WasmOp.Histogram) {
        return {
            normal_hone_ticks: active_profile.normal_grid.toBool(),
            adv_hone_ticks: active_profile.adv_grid.toBool(),
            express_event: active_profile.express_event,

            inp_bound_mats: mats.map((x) => x.bound),
            inp_trade_mats: mats.map((x) => x.tradable),

            inp_market_mats_price: roster_config.mats_prices.toNumArray(),
            inp_trade_mats_price: roster_config.tradable_mats_price.toNumArray(),
            inp_left_mats_price: active_profile.mats_leftover.toNumArray(),

            inp_bound_juice: JUICE_LABELS.map(([weap_key, armor_key]) => [
                active_profile.bound_weap_juice_owned.toNumObj()[weap_key],
                active_profile.bound_weap_juice_owned.toNumObj()[armor_key],
            ]),
            inp_trade_juice: JUICE_LABELS.map(([weap_key, armor_key]) => [
                roster_config.tradable_weap_juice_owned.toNumObj()[weap_key],
                roster_config.tradable_armor_juice_owned.toNumObj()[armor_key],
            ]),

            inp_juice_market_price: JUICE_LABELS.map(([weap_key, armor_key]) => [
                roster_config.weap_juice_prices.toNumObj()[weap_key],
                roster_config.armor_juice_prices.toNumObj()[armor_key],
            ]),
            inp_juice_trade_price: JUICE_LABELS.map(([weap_key, armor_key]) => [
                roster_config.tradable_weap_juice_price.toNumObj()[weap_key],
                roster_config.tradable_armor_juice_price.toNumObj()[armor_key],
            ]),
            inp_juice_left_price: JUICE_LABELS.map(([weap_key, armor_key]) => [
                active_profile.weap_juice_leftover.toNumObj()[weap_key],
                active_profile.armor_juice_leftover.toNumObj()[armor_key],
            ]),
            normal_progress_grid: active_profile.normal_progress_grid,
            normal_state_grid: active_profile.normal_state_grid,
            special_state: active_profile.special_state,
            normal_unlock_grid: active_profile.normal_unlock_grid,
            succeeded_grid: active_profile.succeeded_grid,
            adv_progress_grid: active_profile.adv_progress_grid,
            tier: active_profile.tier,
            min_resolution: active_profile.min_resolution,
            num_threads: 1,
            metric_type: 1,
        }
    }
}
