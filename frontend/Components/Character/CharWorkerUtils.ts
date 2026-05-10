import { useRosterStore } from "@/_stores/RosterConfig";
import { StateBundle, WasmOp } from "@/Utils/Interfaces";
import { grids_to_keyed } from "@/Utils/KeyedUpgrades";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { storeToRefs } from "pinia";

export function grid_change_callback() {
    const { active_profile } = storeToRefs(useRosterStore());

    active_profile.value.keyed_upgrades = grids_to_keyed(
        active_profile.value.normal_grid,
        active_profile.value.adv_grid,
        active_profile.value.keyed_upgrades,
        active_profile.value.tier,
    );
    start_all_workers();
}

export function start_all_workers() {
    const { active_profile } = storeToRefs(useRosterStore());

    // console.log("payload update")
    let payload = build_payload();
    function start_eval_hist(result: StateBundle) {
        if (result === null) return;
        active_profile.value.histogram_worker_bundle.throttled_start(WasmOp.Histogram, build_payload()); // call build payload again here to include the new states
        // active_profile.value.evaluation_worker_bundle.throttled_start(WasmOp.EvaluateAverage, payload)
    }
    active_profile.value.optimizer_worker_bundle.est_progress_percentage = 0;
    if (active_profile.value.auto_start_optimizer) {
        active_profile.value.optimizer_worker_bundle.start(
            WasmOp.OptimizeAverage,
            structuredClone(payload),
            start_eval_hist,
        ); // make sure to clone cos it'll modify the previous payload before it's consumed
    }

    active_profile.value.histogram_worker_bundle.throttled_start(WasmOp.Histogram, structuredClone(payload)); // make sure to clone cos it'll modify the previous payload before it's consumed
    // active_profile.value.evaluation_worker_bundle.throttled_start(WasmOp.EvaluateAverage, payload)
}
