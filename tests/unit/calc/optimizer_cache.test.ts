import { beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { start_all_workers } from "@/Components/Character/CharWorkerUtils";
import { cached_state, store_payload } from "@/WasmInterface/PayloadBuilder";
import { payload_hash } from "@/WasmInterface/OptimizerJobs";
import { WasmOp } from "@/WasmInterface/WasmTypes";
import { fake_state, want_upgrade } from "../income-choices/helpers";

// The calculator restores a saved run of the same payload in place of running the optimizer.

let start: any;
beforeEach(() => {
  setActivePinia(createPinia());
  const store = useRosterStore();
  want_upgrade(store.active_profile);
  const runtime = useRuntimeStore();
  runtime.sync_profiles([store.active_profile.id]);
  start = vi.fn();
  runtime.optimizer.debounced_start = start;
});

describe("optimizer cache", () => {
  it("a saved run of the same payload is restored by a histogram", () => {
    const profile = useRosterStore().active_profile;
    profile.optimizer_cache = cached_state(
      payload_hash(store_payload()),
      fake_state(),
    );
    start_all_workers();
    expect(start.mock.lastCall[0]).toBe(WasmOp.Histogram);
    const out = { state_bundle: { special_state: [7] } };
    start.mock.lastCall[2](out);
    expect(useRuntimeStore().optimizer.result).toBe(out.state_bundle);
  });

  it("another payload runs the optimizer and saves its final state", () => {
    const profile = useRosterStore().active_profile;
    profile.optimizer_cache = cached_state("stale", fake_state());
    start_all_workers();
    expect(start.mock.lastCall[0]).toBe(WasmOp.OptimizeAverage);
    start.mock.lastCall[2]({ ...fake_state(4), adv_cache: null });
    expect(profile.optimizer_cache.hash).toBe(payload_hash(store_payload()));
    expect(profile.optimizer_cache.state.special_state).toEqual([4]);
  });
});
