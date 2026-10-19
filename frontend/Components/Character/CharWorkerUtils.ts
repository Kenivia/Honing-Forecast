import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";

import { grids_to_keyed } from "@/Utils/KeyedUpgrades";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { WasmOp } from "@/WasmInterface/WasmWorker";

export function grid_change_callback(dont_run?: boolean) {
  const profile = useRosterStore().active_profile;

  profile.keyed_upgrades = grids_to_keyed(
    profile.normal_grid,
    profile.adv_grid,
    profile.keyed_upgrades,
    profile.tier,
  );
  if (!dont_run) {
    start_all_workers();
  }
}

export function start_eval_hist() {
  const runtime = useRuntimeStore();
  const profile = useRosterStore().active_profile;
  // call build payload again here to include the new states
  runtime.histogram.throttled_start(
    WasmOp.Histogram,
    build_payload(profile.optimizer_override),
  );
}

export function start_all_workers() {
  const runtime = useRuntimeStore();
  const profile = useRosterStore().active_profile;

  runtime.optimizer.est_progress_percentage = 0;
  if (profile.auto_start_optimizer) {
    runtime.optimizer.debounced_start(
      WasmOp.OptimizeAverage,
      build_payload(),
      (result) => {
        const { adv_cache, ...rest } = result;
        runtime.adv_cache = adv_cache;
        runtime.optimizer.result = rest;
        start_eval_hist();
      },
    );
  }

  start_eval_hist();
}
