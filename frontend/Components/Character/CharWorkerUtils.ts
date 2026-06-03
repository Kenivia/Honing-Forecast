import { useRosterStore } from "@/Stores/RosterConfig";

import { grids_to_keyed } from "@/Utils/KeyedUpgrades";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { StateBundle, WasmOp } from "@/WasmInterface/WasmWorker";
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
export function start_eval_hist() {
  const { active_profile } = storeToRefs(useRosterStore());
  active_profile.value.histogram_worker_bundle.throttled_start(
    WasmOp.Histogram,
    build_payload(active_profile.value.optimizer_override),
  ); // call build payload again here to include the new states

  // active_profile.value.evaluation_worker_bundle.throttled_start(WasmOp.EvaluateAverage, payload)
}
export function start_all_workers() {
  const { active_profile } = storeToRefs(useRosterStore());

  // console.log("payload update")

  active_profile.value.optimizer_worker_bundle.est_progress_percentage = 0;
  if (active_profile.value.auto_start_optimizer) {
    active_profile.value.optimizer_worker_bundle.debounced_start(
      WasmOp.OptimizeAverage,
      build_payload(),
      start_eval_hist,
    ); // make sure to clone cos it'll modify the previous payload before it's consumed
  }

  active_profile.value.histogram_worker_bundle.throttled_start(
    WasmOp.Histogram,
    build_payload(active_profile.value.optimizer_override),
  ); // make sure to clone cos it'll modify the previous payload before it's consumed
  // active_profile.value.evaluation_worker_bundle.throttled_start(WasmOp.EvaluateAverage, payload)
}
