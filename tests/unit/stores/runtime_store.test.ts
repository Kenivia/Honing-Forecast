import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { useRosterStore } from "@/Stores/RosterConfig";
import { rich_config } from "./fixtures";

vi.mock("@/WasmInterface/WorkerBundle", () => ({
  create_worker_bundle: () => ({ cancel: vi.fn() }),
}));

let runtime: ReturnType<typeof useRuntimeStore>;
let ids: string[];
beforeEach(() => {
  setActivePinia(createPinia());
  const roster = useRosterStore();
  roster.replace_config(rich_config());
  ids = roster.all_profiles.map((p) => p.id);
  runtime = useRuntimeStore();
});

describe("sync_profiles", () => {
  it("creates one optimizer / histogram pair per id", () => {
    runtime.sync_profiles(ids);
    expect(Object.keys(runtime.per_char).sort()).toEqual([...ids].sort());
    const pair = runtime.for_char(ids[0]);
    expect(pair.optimizer).not.toBe(pair.histogram);
    expect(pair).not.toBe(runtime.for_char(ids[1]));
  });

  it("keeps existing pairs and cancels and drops removed ones", () => {
    runtime.sync_profiles(ids);
    const kept = runtime.for_char(ids[0]);
    const dropped = runtime.for_char(ids[1]);
    runtime.sync_profiles([ids[0], ids[2], "new"]);
    expect(runtime.for_char(ids[0])).toBe(kept);
    expect(runtime.for_char(ids[1])).toBeUndefined();
    expect(runtime.for_char("new")).toBeDefined();
    expect(dropped.optimizer.cancel).toHaveBeenCalled();
    expect(dropped.histogram.cancel).toHaveBeenCalled();
    expect(kept.optimizer.cancel).not.toHaveBeenCalled();
  });
});

describe("active", () => {
  it("is the active profile's pair, or the idle stand-in before a sync", () => {
    expect(runtime.active).toBe(runtime.idle);
    runtime.sync_profiles(ids);
    expect(runtime.optimizer).toBe(runtime.for_char(ids[2]).optimizer);
    useRosterStore().switch_profile(0);
    expect(runtime.histogram).toBe(runtime.for_char(ids[0]).histogram);
  });

  it("cancel_active cancels only the active pair", () => {
    runtime.sync_profiles(ids);
    runtime.cancel_active();
    expect(runtime.for_char(ids[2]).optimizer.cancel).toHaveBeenCalledTimes(1);
    expect(runtime.for_char(ids[2]).histogram.cancel).toHaveBeenCalledTimes(1);
    expect(runtime.for_char(ids[0]).optimizer.cancel).not.toHaveBeenCalled();
  });

  it("a rename keeps the pair", () => {
    runtime.sync_profiles(ids);
    const before = runtime.active;
    useRosterStore().active_profile.char_name = "Renamed";
    expect(runtime.active).toBe(before);
  });
});

describe("cropper", () => {
  it("is created once", () => {
    expect(runtime.cropper).toBeNull();
    const first = runtime.ensure_cropper();
    expect(runtime.ensure_cropper()).toBe(first);
  });
});
