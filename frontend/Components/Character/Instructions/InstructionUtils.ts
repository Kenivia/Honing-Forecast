import { useRosterStore } from "@/Stores/RosterConfig";
import { AdvOverride, NormalOverride } from "@/WasmInterface/PayloadBuilder";
import { StateBundle } from "@/WasmInterface/WasmWorker";
import { storeToRefs } from "pinia";

export function get_any_overwritten(): boolean {
  const { active_profile } = storeToRefs(useRosterStore());
  return (
    active_profile.value.optimizer_override.normal.juice !==
      NormalOverride.Optimizer ||
    active_profile.value.optimizer_override.normal.book !==
      NormalOverride.Optimizer ||
    active_profile.value.optimizer_override.advanced.juice !==
      AdvOverride.Optimizer ||
    active_profile.value.optimizer_override.advanced.scroll !==
      AdvOverride.Optimizer ||
    active_profile.value.optimizer_override.special.optimizer !== true
  );
}
export function get_optimizer_working(): boolean {
  const { active_profile } = storeToRefs(useRosterStore());
  return active_profile.value.optimizer_worker_bundle.status === "busy";
}
export function get_relevant_result(any_overwritten: boolean): StateBundle {
  const { active_profile } = storeToRefs(useRosterStore());
  return any_overwritten
    ? active_profile.value.histogram_worker_bundle.result.state_bundle
    : active_profile.value.optimizer_worker_bundle.result;
}
