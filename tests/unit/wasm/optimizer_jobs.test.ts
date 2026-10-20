import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeWorker, install_fake_worker } from "./fakes";
import { WasmOp } from "@/WasmInterface/WasmTypes";

vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
// the histogram payload is tagged with the state it was built from
vi.mock("@/WasmInterface/PayloadBuilder", () => ({
  states_payload: (payload: any, _profile: any, state: any) => ({
    ...payload,
    from_state: state,
  }),
}));

type Jobs = typeof import("@/WasmInterface/OptimizerJobs");
let jobs: Jobs;

beforeEach(async () => {
  install_fake_worker();
  vi.spyOn(console, "log").mockImplementation(() => {});
  vi.spyOn(console, "error").mockImplementation(() => {});
  vi.resetModules();
  jobs = await import("@/WasmInterface/OptimizerJobs");
});
afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

const job = (hash: string) => ({
  hash,
  payload: { name: hash } as any,
  profile: {} as any,
  store: vi.fn(),
});

// the workers whose last post is an optimizer run of `name`
const optimizing = (name: string) =>
  FakeWorker.all.filter(
    (w) =>
      !w.terminated &&
      w.last()?.wasm_op === WasmOp.OptimizeAverage &&
      w.last().payload.name === name,
  );
const histogramming = (name: string) =>
  FakeWorker.all.filter(
    (w) =>
      !w.terminated &&
      w.last()?.wasm_op === WasmOp.Histogram &&
      w.last().payload.name === name,
  );

// the optimizer's result, then the histogram's
function finish(
  name: string,
  state: any = { s: name },
  out: any = { o: name },
) {
  optimizing(name)[0].reply(state);
  histogramming(name)[0].reply(out);
}

describe("hash", () => {
  it("is stable and tells texts apart", () => {
    expect(jobs.hash("abc")).toBe(jobs.hash("abc"));
    expect(jobs.hash("abc")).not.toBe(jobs.hash("abd"));
    expect(jobs.hash("")).toMatch(/^[0-9a-z]+$/);
  });
});

describe("queue_jobs", () => {
  it("runs two at a time, each an optimizer run then a histogram of its state", () => {
    const [a, b, c] = [job("a"), job("b"), job("c")];
    jobs.queue_jobs([a, b, c]);
    expect(jobs.jobs_left.value).toBe(3);
    expect(optimizing("a")).toHaveLength(1);
    expect(optimizing("b")).toHaveLength(1);
    expect(optimizing("c")).toHaveLength(0);

    optimizing("a")[0].reply({ s: "a" });
    const hist = histogramming("a")[0].last();
    expect(hist.payload.from_state).toEqual({ s: "a" });
    expect(a.store).not.toHaveBeenCalled();

    histogramming("a")[0].reply({ o: "a" });
    expect(a.store).toHaveBeenCalledWith({ o: "a" }, { s: "a" });
    // the freed slot takes the next
    expect(optimizing("c")).toHaveLength(1);
    expect(jobs.jobs_left.value).toBe(2);

    finish("b");
    finish("c");
    expect(jobs.jobs_left.value).toBe(0);
    expect(b.store).toHaveBeenCalledTimes(1);
    expect(c.store).toHaveBeenCalledTimes(1);
  });

  it("ignores interim states", () => {
    jobs.queue_jobs([job("a")]);
    optimizing("a")[0].interim({ s: "interim" });
    expect(histogramming("a")).toHaveLength(0);
    expect(optimizing("a")).toHaveLength(1);
  });

  it("replaces the queue but lets running jobs carry on", () => {
    const [a, b, c, d] = [job("a"), job("b"), job("c"), job("d")];
    jobs.queue_jobs([a, b, c]);
    jobs.queue_jobs([job("a"), d]);
    // a is not started twice, c is dropped, d waits
    expect(
      FakeWorker.all.filter((w) => w.last().payload.name === "a"),
    ).toHaveLength(1);
    expect(jobs.jobs_left.value).toBe(3);
    finish("b");
    expect(b.store).toHaveBeenCalled();
    expect(optimizing("d")).toHaveLength(1);
    expect(optimizing("c")).toHaveLength(0);
    finish("a");
    expect(a.store).toHaveBeenCalled();
    finish("d");
    expect(jobs.jobs_left.value).toBe(0);
  });

  it("stop_jobs cancels everything and both slots are free again", () => {
    const [a, b] = [job("a"), job("b")];
    jobs.queue_jobs([a, b, job("c")]);
    jobs.stop_jobs();
    expect(jobs.jobs_left.value).toBe(0);
    expect(FakeWorker.all.every((w) => w.terminated)).toBe(true);
    expect(a.store).not.toHaveBeenCalled();
    jobs.queue_jobs([job("x"), job("y")]);
    expect(optimizing("x")).toHaveLength(1);
    expect(optimizing("y")).toHaveLength(1);
  });

  it("a hash queued twice runs once and both slots stay usable", () => {
    jobs.queue_jobs([job("a"), job("a")]);
    expect(optimizing("a")).toHaveLength(1);
    expect(jobs.jobs_left.value).toBe(1);
    finish("a");
    expect(jobs.jobs_left.value).toBe(0);
    jobs.queue_jobs([job("x"), job("y")]);
    expect(optimizing("x")).toHaveLength(1);
    expect(optimizing("y")).toHaveLength(1);
  });

  it("one run of a hash stores into every job with that hash", () => {
    const [a, other_a, b] = [job("a"), job("a"), job("b")];
    jobs.queue_jobs([a, b, other_a]);
    finish("a");
    expect(a.store).toHaveBeenCalledWith({ o: "a" }, { s: "a" });
    expect(other_a.store).toHaveBeenCalledWith({ o: "a" }, { s: "a" });
    expect(b.store).not.toHaveBeenCalled();
    finish("b");
    // a job of that hash queued while it runs is served too
    const [c, later_c] = [job("c"), job("c")];
    jobs.queue_jobs([c]);
    jobs.queue_jobs([later_c]);
    finish("c");
    expect(later_c.store).toHaveBeenCalledOnce();
    expect(jobs.jobs_left.value).toBe(0);
  });

  it("a failed optimizer or histogram run is dropped and frees its slot", () => {
    const [a, b] = [job("a"), job("b")];
    jobs.queue_jobs([a, b, job("c"), job("d")]);
    optimizing("a")[0].fail();
    expect(optimizing("c")).toHaveLength(1);
    optimizing("b")[0].reply({ s: "b" });
    histogramming("b")[0].fail();
    expect(optimizing("d")).toHaveLength(1);
    expect(a.store).not.toHaveBeenCalled();
    expect(b.store).not.toHaveBeenCalled();
    expect(jobs.jobs_left.value).toBe(2);
  });
});
