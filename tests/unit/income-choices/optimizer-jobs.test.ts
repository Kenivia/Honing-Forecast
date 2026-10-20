import { beforeEach, describe, expect, it, vi } from "vitest";
import { fake_out, fake_state } from "./helpers";

// The shared queue under IncomeJobs and ChoiceJobs, with fake worker bundles.
interface Call {
  op: number;
  payload: any;
  callback: (result: any) => void;
}
interface FakeBundle {
  status: "idle" | "busy";
  calls: Call[];
  cancelled: number;
  debounced_start: (op: number, payload: any, callback: any) => void;
  cancel: () => void;
}
const bundles: FakeBundle[] = [];

vi.mock("@/WasmInterface/PayloadBuilder", () => ({
  states_payload: (payload: any, _profile: any, state: any) => ({
    ...payload,
    state,
  }),
}));
vi.mock("@/WasmInterface/WorkerBundle", () => ({
  create_worker_bundle: () => {
    const bundle: FakeBundle = {
      status: "idle",
      calls: [],
      cancelled: 0,
      debounced_start(op, payload, callback) {
        bundle.status = "busy";
        bundle.calls.push({ op, payload, callback });
      },
      cancel() {
        bundle.cancelled++;
        bundle.status = "idle";
      },
    };
    bundles.push(bundle);
    return bundle;
  },
}));

let jobs: typeof import("@/WasmInterface/OptimizerJobs");
beforeEach(async () => {
  bundles.length = 0;
  vi.resetModules();
  jobs = await import("@/WasmInterface/OptimizerJobs");
});

const job = (name: string, store = vi.fn()) => ({
  hash: name,
  payload: { name } as any,
  profile: {} as any,
  store,
});
const last = (bundle: FakeBundle) => bundle.calls[bundle.calls.length - 1];
// the optimizer finishing with `state`, then its histogram with `out`
function finish(
  bundle: FakeBundle,
  state = fake_state(),
  out = fake_out(1, 2),
) {
  bundle.status = "idle";
  last(bundle).callback(state);
  bundle.status = "idle";
  last(bundle).callback(out);
}

describe("OptimizerJobs", () => {
  it("runs two at a time, in order, each an optimizer run then a histogram", () => {
    const a = job("a");
    jobs.queue_jobs([a, job("b"), job("c")]);
    expect(bundles.map((b) => b.calls.map((c) => c.payload.name))).toEqual([
      ["a"],
      ["b"],
    ]);
    expect(jobs.jobs_left.value).toBe(3);

    const state = fake_state(7);
    finish(bundles[0], state, fake_out(5, 6));
    expect(bundles[0].calls.map((c) => c.op)).toEqual([0, 1, 0]);
    // the histogram gets the optimizer's state
    expect(bundles[0].calls[1].payload.state).toBe(state);
    expect(a.store).toHaveBeenCalledWith(fake_out(5, 6), state);
    expect(last(bundles[0]).payload.name).toBe("c");
    expect(jobs.jobs_left.value).toBe(2);
  });

  it("ignores interim states while the optimizer is busy", () => {
    jobs.queue_jobs([job("a")]);
    last(bundles[0]).callback(fake_state());
    expect(bundles[0].calls).toHaveLength(1);
  });

  it("a new queue replaces the old, running ones carry on and are not queued again", () => {
    const a = job("a");
    jobs.queue_jobs([a, job("b"), job("c")]);
    jobs.queue_jobs([job("a"), job("d")]);
    expect(jobs.jobs_left.value).toBe(3);
    finish(bundles[0]);
    expect(a.store).toHaveBeenCalledOnce();
    expect(last(bundles[0]).payload.name).toBe("d");
  });

  it("stopping cancels both workers and drops late results", () => {
    const a = job("a");
    jobs.queue_jobs([a, job("b"), job("c")]);
    jobs.stop_jobs();
    expect(bundles.map((b) => b.cancelled)).toEqual([1, 1]);
    expect(jobs.jobs_left.value).toBe(0);
    finish(bundles[0]);
    expect(a.store).not.toHaveBeenCalled();
    // and the workers take new jobs
    jobs.queue_jobs([job("e")]);
    expect(last(bundles[0]).payload.name).toBe("e");
  });

  // a hash queued twice runs once
  it("a job queued twice does not leave a worker stuck", () => {
    jobs.queue_jobs([job("a"), job("a"), job("b"), job("c")]);
    finish(bundles[0]);
    finish(bundles[1]);
    const names = bundles.flatMap((b) => b.calls.map((c) => c.payload.name));
    expect(names).toContain("c");
  });

  it("hash is stable and differs on a change", () => {
    expect(jobs.hash("abc")).toBe(jobs.hash("abc"));
    expect(jobs.hash("abc")).not.toBe(jobs.hash("abd"));
  });
});
