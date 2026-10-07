import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { FLOAT_TOL } from "@/Utils/Constants";
import { AdvOverride, NormalOverride } from "@/WasmInterface/PayloadBuilder";
import { StateBundle } from "@/WasmInterface/WasmWorker";

export function get_any_overwritten(): boolean {
  const runtime = useRuntimeStore();
  const { optimizer_override } = useRosterStore().active_profile;
  return (
    (optimizer_override.normal.juice !== NormalOverride.Optimizer ||
      optimizer_override.normal.book !== NormalOverride.Optimizer ||
      optimizer_override.advanced.juice !== AdvOverride.Optimizer ||
      optimizer_override.advanced.scroll !== AdvOverride.Optimizer ||
      optimizer_override.special.optimizer !== true) &&
    // the payload's optimizer_plan, so this is the metric the optimizer maximised
    Math.abs(
      runtime.optimizer.result.metric -
        runtime.histogram.result?.metric_per_plan[1],
    ) > FLOAT_TOL
  );
}

export function get_optimizer_working(): boolean {
  return useRuntimeStore().optimizer.status === "busy";
}

export function get_relevant_result(any_overwritten: boolean): StateBundle {
  const runtime = useRuntimeStore();
  return any_overwritten
    ? runtime.histogram.result.state_bundle
    : runtime.optimizer.result;
}
