import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { FakeWorker, install_fake_worker } from "./fakes";

vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));

import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import {
  grid_change_callback,
  start_all_workers,
} from "@/Components/Character/CharWorkerUtils";
import { WasmOp } from "@/WasmInterface/WasmTypes";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";

let store: ReturnType<typeof useRosterStore>;
let runtime: ReturnType<typeof useRuntimeStore>;

beforeEach(() => {
  install_fake_worker();
  vi.useFakeTimers();
  vi.spyOn(console, "log").mockImplementation(() => {});
  setActivePinia(createPinia());
  store = useRosterStore();
  runtime = useRuntimeStore();
  runtime.sync_profiles([store.active_profile.id]);
});
afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

const posts = (op: WasmOp) =>
  FakeWorker.all.flatMap((w) => w.posted).filter((m) => m.wasm_op === op);
const worker_of = (op: WasmOp) =>
  FakeWorker.all.filter((w) => !w.terminated && w.last()?.wasm_op === op)[0];

describe("grid_change_callback", () => {
  it("rebuilds the keyed upgrades from the grids, and runs nothing when told not to", () => {
    const profile = store.active_profile;
    profile.normal_grid[0][5] = UpgradeStatus.Want;
    grid_change_callback(true);
    expect(Object.keys(profile.keyed_upgrades)).toEqual(["0,5,true,0"]);
    vi.advanceTimersByTime(1000);
    expect(FakeWorker.all).toHaveLength(0);
  });
});

describe("start_all_workers", () => {
  it("runs the histogram at once and the optimizer after the debounce", () => {
    start_all_workers();
    expect(posts(WasmOp.Histogram)).toHaveLength(1);
    expect(posts(WasmOp.OptimizeAverage)).toHaveLength(0);
    vi.advanceTimersByTime(200);
    expect(posts(WasmOp.OptimizeAverage)).toHaveLength(1);
    // on the active character's own bundles
    expect(runtime.optimizer).toBe(
      runtime.for_char(store.active_profile.id).optimizer,
    );
  });

  it("skips the optimizer when auto-start is off", () => {
    store.active_profile.auto_start_optimizer = false;
    start_all_workers();
    vi.advanceTimersByTime(1000);
    expect(posts(WasmOp.OptimizeAverage)).toHaveLength(0);
    expect(posts(WasmOp.Histogram)).toHaveLength(1);
  });

  it("an optimizer result is stored, its cache apart, and reruns the histogram with its state", () => {
    start_all_workers();
    worker_of(WasmOp.Histogram).reply({ juice_info: null });
    vi.advanceTimersByTime(200);
    const cache = new Map([["c", 1]]);
    worker_of(WasmOp.OptimizeAverage).reply({
      special_state: [0],
      chest_states: new Map(),
      upgrade_arr: [],
      adv_cache: cache,
    });
    expect(runtime.adv_cache).toEqual(cache);
    expect(runtime.optimizer.result).not.toHaveProperty("adv_cache");
    expect(runtime.optimizer.result.special_state).toEqual([0]);
    const hist = posts(WasmOp.Histogram);
    expect(hist).toHaveLength(2);
    expect(hist[1].payload.special_state).toEqual([0]);
  });

  it("an interim state also reruns the histogram", () => {
    start_all_workers();
    worker_of(WasmOp.Histogram).reply({});
    vi.advanceTimersByTime(200);
    worker_of(WasmOp.OptimizeAverage).interim({
      special_state: [0],
      upgrade_arr: [],
    });
    expect(runtime.optimizer.result.special_state).toEqual([0]);
    expect(posts(WasmOp.Histogram)).toHaveLength(2);
  });

  // Rust sends interim states with the cache emptied
  it("an interim state keeps the advanced honing cache", () => {
    const cache = new Map([["c", 1]]);
    runtime.adv_cache = cache;
    start_all_workers();
    vi.advanceTimersByTime(200);
    worker_of(WasmOp.OptimizeAverage).interim({
      special_state: [0],
      upgrade_arr: [],
      adv_cache: new Map(),
    });
    expect(runtime.adv_cache).toEqual(cache);
  });
});
