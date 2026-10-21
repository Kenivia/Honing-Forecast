import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";

import { grids_to_keyed } from "@/Utils/KeyedUpgrades";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { WasmOp } from "@/WasmInterface/WasmTypes";
import { ALL_LABELS } from "@/Utils/Constants";

// plans[0] of every payload
const ALL_WORTHLESS_PLAN = 0;

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
  eval_hist(useRuntimeStore().optimizer.status === "idle");
}

// The gold distribution is slow, so it is only drawn for a state the optimizer is done with.
function eval_hist(with_gold: boolean) {
  const runtime = useRuntimeStore();
  const profile = useRosterStore().active_profile;
  // call build payload again here to include the new states
  runtime.histogram.throttled_start(WasmOp.Histogram, {
    ...build_payload(profile.optimizer_override),
    dist_plan: with_gold ? ALL_WORTHLESS_PLAN : null,
  });
}

// The gold one more unit of each enabled material saves, from a finished optimizer result.
function start_marginal() {
  const profile = useRosterStore().active_profile;
  const enabled = profile.bound_budgets.enabled;
  useRuntimeStore().marginal.debounced_start(
    WasmOp.MarginalValue,
    {
      payload: build_payload(),
      labels: ALL_LABELS[profile.tier].filter(
        (label) => label !== "Gold" && enabled[label] !== false,
      ),
    },
    undefined,
    0,
  );
}

export function start_all_workers() {
  const runtime = useRuntimeStore();
  const profile = useRosterStore().active_profile;

  runtime.optimizer.est_progress_percentage = 0;
  if (profile.auto_start_optimizer) {
    runtime.marginal.cancel();
    runtime.optimizer.debounced_start(
      WasmOp.OptimizeAverage,
      build_payload(),
      (result) => {
        if (!result) return;
        const { adv_cache, ...rest } = result;
        // interim states come with the cache emptied
        if (adv_cache?.size) runtime.adv_cache = adv_cache;
        runtime.optimizer.result = rest;
        start_eval_hist();
        if (runtime.optimizer.status === "idle") start_marginal();
      },
    );
  }

  // the optimizer is about to run when it starts on its own
  eval_hist(!profile.auto_start_optimizer);
}
