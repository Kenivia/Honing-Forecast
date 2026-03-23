import { CharProfile } from "@/Stores/CharacterProfile"
import { RosterConfig } from "@/Stores/RosterConfig"
import { grids_to_keyed, StateBundle } from "@/Utils/Interfaces"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { build_material_info, build_payload } from "@/WasmInterface/payload"
import { onWatcherCleanup } from "vue"

export function grid_change_callback(active_profile: CharProfile, roster_config: RosterConfig) {
    active_profile.keyed_upgrades = grids_to_keyed(active_profile.normal_grid, active_profile.adv_grid, active_profile.keyed_upgrades, active_profile.tier)

    start_all_workers(active_profile, roster_config)
}

export function start_all_workers(active_profile: CharProfile, roster_config: RosterConfig) {
    // console.log("payload update")
    let payload = build_payload(WasmOp.OptimizeAverage, active_profile, roster_config)
    function start_eval_hist(result: StateBundle) {
        if (result === null) return
        active_profile.histogram_worker_bundle.throttled_start(WasmOp.Histogram, build_payload(WasmOp.Histogram, active_profile, roster_config))
        active_profile.evaluation_worker_bundle.throttled_start(WasmOp.EvaluateAverage, build_payload(WasmOp.EvaluateAverage, active_profile, roster_config))
    }
    active_profile.optimizer_worker_bundle.est_progress_percentage = 0
    if (active_profile.auto_start_optimizer) {
        active_profile.optimizer_worker_bundle.start(WasmOp.OptimizeAverage, payload, start_eval_hist)
    }
    payload.material_info = build_material_info(WasmOp.Histogram, active_profile, roster_config)
    active_profile.histogram_worker_bundle.throttled_start(WasmOp.Histogram, payload)

    payload.material_info = build_material_info(WasmOp.EvaluateAverage, active_profile, roster_config)
    active_profile.evaluation_worker_bundle.throttled_start(WasmOp.EvaluateAverage, payload)
}
