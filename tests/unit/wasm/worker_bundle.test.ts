import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeWorker, install_fake_worker, live } from "./fakes";

vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));

import { create_worker_bundle } from "@/WasmInterface/WorkerBundle";
import { WasmOp } from "@/WasmInterface/WasmTypes";

beforeEach(() => {
  install_fake_worker();
  vi.useFakeTimers();
  vi.spyOn(console, "log").mockImplementation(() => {});
  vi.spyOn(console, "error").mockImplementation(() => {});
});
afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("debounced_start", () => {
  it("launches once, with the last payload, after the debounce", () => {
    const b = create_worker_bundle();
    b.debounced_start(WasmOp.OptimizeAverage, { n: 1 });
    vi.advanceTimersByTime(150);
    b.debounced_start(WasmOp.OptimizeAverage, { n: 2 });
    vi.advanceTimersByTime(199);
    expect(FakeWorker.all).toHaveLength(0);
    expect(b.status).toBe("idle");
    vi.advanceTimersByTime(1);
    expect(FakeWorker.all).toHaveLength(1);
    expect(FakeWorker.all[0].posted).toHaveLength(1);
    expect(FakeWorker.all[0].last()).toMatchObject({
      wasm_op: WasmOp.OptimizeAverage,
      payload: { n: 2 },
    });
    expect(b.status).toBe("busy");
    expect(b.run_counter).toBe(1);
  });

  it("a debounce of 0 launches at once", () => {
    const b = create_worker_bundle();
    b.debounced_start(WasmOp.Histogram, {}, undefined, 0);
    expect(FakeWorker.all).toHaveLength(1);
  });

  it("a new run terminates the one in progress, and its result terminates its worker", () => {
    const b = create_worker_bundle();
    const callback = vi.fn();
    b.debounced_start(WasmOp.OptimizeAverage, { n: 1 }, callback, 0);
    b.debounced_start(WasmOp.OptimizeAverage, { n: 2 }, callback, 0);
    const [first, second] = FakeWorker.all;
    expect(first.terminated).toBe(true);
    expect(second.terminated).toBe(false);
    second.reply({ metric: 3 });
    expect(second.terminated).toBe(true);
    expect(b.status).toBe("idle");
    expect(b.result).toEqual({ metric: 3 });
    expect(b.est_progress_percentage).toBe(100);
    expect(callback).toHaveBeenCalledWith({ metric: 3 }, expect.anything());
    // every run gets a fresh worker
    b.debounced_start(WasmOp.OptimizeAverage, {}, undefined, 0);
    expect(FakeWorker.all).toHaveLength(3);
  });

  it("an interim state reaches the callback while still busy", () => {
    const b = create_worker_bundle();
    const callback = vi.fn();
    b.debounced_start(WasmOp.OptimizeAverage, {}, callback, 0);
    const w = FakeWorker.all[0];
    w.interim({ special_state: [1] }, 40);
    expect(callback).toHaveBeenCalledWith({ special_state: [1] });
    expect(b.status).toBe("busy");
    expect(b.est_progress_percentage).toBe(40);
    // progress without a state does not call back
    w.interim(undefined, 60);
    expect(callback).toHaveBeenCalledTimes(1);
    expect(b.est_progress_percentage).toBe(60);
  });

  it("cancel drops a pending launch and terminates the running worker", () => {
    const b = create_worker_bundle();
    b.debounced_start(WasmOp.OptimizeAverage, {}, undefined, 0);
    b.debounced_start(WasmOp.OptimizeAverage, {});
    b.cancel();
    vi.advanceTimersByTime(1000);
    expect(FakeWorker.all).toHaveLength(1);
    expect(live()).toHaveLength(0);
    expect(b.status).toBe("idle");
  });

  it("cancel_and_clear_prev_result also forgets the result", () => {
    const b = create_worker_bundle();
    b.debounced_start(WasmOp.OptimizeAverage, {}, undefined, 0);
    FakeWorker.all[0].reply({ x: 1 });
    b.cancel();
    expect(b.result).toEqual({ x: 1 });
    b.cancel_and_clear_prev_result();
    expect(b.result).toBeNull();
    expect(b.est_progress_percentage).toBe(0);
  });

  it("a worker error goes idle with the message and the next run gets a new worker", () => {
    const b = create_worker_bundle();
    b.debounced_start(WasmOp.OptimizeAverage, {}, undefined, 0);
    FakeWorker.all[0].fail("panic");
    expect(b.error).toBe("panic");
    expect(b.status).toBe("idle");
    b.debounced_start(WasmOp.OptimizeAverage, {}, undefined, 0);
    expect(FakeWorker.all).toHaveLength(2);
    expect(b.error).toBeNull();
  });

  it("an op that throws in the worker fails the same way and calls back with null", () => {
    const b = create_worker_bundle();
    const callback = vi.fn();
    b.debounced_start(WasmOp.Histogram, {}, callback, 0, false);
    FakeWorker.all[0].crash("unreachable");
    expect(callback).toHaveBeenCalledWith(null);
    expect(b.error).toBe("unreachable");
    expect(b.status).toBe("idle");
    expect(FakeWorker.all[0].terminated).toBe(true);
    expect(console.error).toHaveBeenCalled();
  });

  it("a worker error terminates the worker", () => {
    const b = create_worker_bundle();
    b.debounced_start(WasmOp.OptimizeAverage, {}, undefined, 0);
    FakeWorker.all[0].fail();
    expect(FakeWorker.all[0].terminated).toBe(true);
  });
});

describe("throttled_start", () => {
  it("runs the first at once and only the latest of those that came while busy", () => {
    const b = create_worker_bundle();
    b.throttled_start(WasmOp.Histogram, { n: 1 });
    b.throttled_start(WasmOp.Histogram, { n: 2 });
    b.throttled_start(WasmOp.Histogram, { n: 3 });
    const w = FakeWorker.all[0];
    expect(w.posted.map((m) => m.payload.n)).toEqual([1]);
    vi.advanceTimersByTime(300);
    expect(w.posted).toHaveLength(1);
    w.reply({ r: 1 });
    expect(w.posted.map((m) => m.payload.n)).toEqual([1, 3]);
    // reused, never cancelled midway
    expect(FakeWorker.all).toHaveLength(1);
    expect(w.terminated).toBe(false);
  });

  it("waits out 200 ms between launches even when the result is quick", () => {
    const b = create_worker_bundle();
    b.throttled_start(WasmOp.Histogram, { n: 1 });
    const w = FakeWorker.all[0];
    vi.advanceTimersByTime(50);
    w.reply({});
    b.throttled_start(WasmOp.Histogram, { n: 2 });
    expect(w.posted).toHaveLength(1);
    vi.advanceTimersByTime(149);
    expect(w.posted).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(w.posted.map((m) => m.payload.n)).toEqual([1, 2]);
  });

  it("an idle bundle past the gap launches at once", () => {
    const b = create_worker_bundle();
    b.throttled_start(WasmOp.Histogram, { n: 1 });
    FakeWorker.all[0].reply({});
    vi.advanceTimersByTime(200);
    b.throttled_start(WasmOp.Histogram, { n: 2 });
    expect(FakeWorker.all[0].posted).toHaveLength(2);
  });

  it("cancel drops what is pending", () => {
    const b = create_worker_bundle();
    b.throttled_start(WasmOp.Histogram, { n: 1 });
    b.throttled_start(WasmOp.Histogram, { n: 2 });
    b.cancel();
    vi.advanceTimersByTime(1000);
    expect(FakeWorker.all[0].posted).toHaveLength(1);
    // and the next one goes at once on a new worker
    b.throttled_start(WasmOp.Histogram, { n: 3 });
    expect(FakeWorker.all).toHaveLength(2);
    expect(FakeWorker.all[1].last().payload.n).toBe(3);
  });

  it("what was pending at a worker error runs when the gap ends", () => {
    const b = create_worker_bundle();
    b.throttled_start(WasmOp.Histogram, { n: 1 });
    b.throttled_start(WasmOp.Histogram, { n: 2 });
    FakeWorker.all[0].fail();
    vi.advanceTimersByTime(200);
    expect(FakeWorker.all).toHaveLength(2);
    expect(FakeWorker.all[1].last().payload.n).toBe(2);
  });
});
