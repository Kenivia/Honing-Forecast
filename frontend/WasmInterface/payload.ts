import { JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants"
import { WasmOp } from "./js_to_wasm"

import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { uesRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES } from "@/Utils/Constants"
import { iconPath } from "@/Utils/Helpers"
import { Upgrade, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { computed } from "vue"
import InstructionRow from "./InstructionRow.vue"

const profile_store = useProfilesStore()
const active_profile: CharProfile = profile_store.activeProfile()

const roster_store = uesRosterStore()

const optimizer_worker = active_profile.optimizer_worker_bundle
const optimizer_busy = optimizer_worker.status === "running" || optimizer_worker.status === "pending"
const has_run_optimizer = active_profile.has_run_optimizer
const auto_start_optimizer = active_profile.auto_start_optimizer
const optimizer_progress = optimizer_worker.est_progress_percentage

function parseFloatZero(input: string) {
    const out = Number.parseFloat(input)
    return Number.isFinite(out) ? out : 0
}

export function buildPayload(wasm_op: WasmOp) {
    return {
        mats_budget: MATS_LABELS.slice(0, 7).map((label) => parseFloatZero(mats.owned[label] || "0")),

        express_event: active_profile.express_event,
        bucket_count: Math.max(2, Math.min(1000, Math.floor(Number(bucketCount) || 2))),
        user_price_arr: MATS_LABELS.slice(0, 7).map((label) => parseFloatZero(mats.prices[label] || "0")),
        data_size: Math.max(1000, Math.floor(Number(dataSize) || 0)),
        inp_leftover_values: MATS_LABELS.slice(0, 7).map((label) => parseFloatZero(mats.leftover[label] || "0")),
        juice_books_budget: JUICE_LABELS.map((labels) => [parseFloatZero(juice.weapon.owned[labels[0]]), parseFloatZero(juice.armor.owned[labels[1]])]),
        juice_prices: JUICE_LABELS.map((labels) => [parseFloatZero(juice.weapon.prices[labels[0]]), parseFloatZero(juice.armor.prices[labels[1]])]),
        inp_leftover_juice_values: JUICE_LABELS.map((labels) => [
            parseFloatZero(juice.weapon.leftover[labels[0]]),
            parseFloatZero(juice.armor.leftover[labels[1]]),
        ]),
        progress_grid: progressGrid,
        unlocked_grid: unlockGrid,
        succeeded_grid: succeededGrid,
        state_grid: stateBundleGrid,
        special_state: specialState,
        min_resolution: Math.max(1, Math.min(219, Math.floor(minResolution || 1))),
        num_threads: navigator.hardwareConcurrency,
        metric_type: metricType,
        normal_hone_ticks: topGrid,
        adv_hone_ticks: bottomGrid,
        tier: 0,
    }
}
