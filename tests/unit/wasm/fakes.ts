import { vi } from "vitest";

// A Worker the test drives: it records posts and replies only when told to.
export class FakeWorker {
  static all: FakeWorker[] = [];
  posted: any[] = [];
  terminated = false;
  onmessage: ((e: { data: any }) => void) | null = null;
  onerror: ((e: { message: string }) => void) | null = null;
  constructor() {
    FakeWorker.all.push(this);
  }
  postMessage(msg: any) {
    this.posted.push(msg);
  }
  terminate() {
    this.terminated = true;
  }
  addEventListener() {}
  removeEventListener() {}
  last() {
    return this.posted[this.posted.length - 1];
  }
  reply(result: any) {
    this.onmessage?.({ data: { type: "result", result, timings: {} } });
  }
  interim(state_bundle: any, est_progress_percentage = 50) {
    this.onmessage?.({
      data: {
        type: "intermediate_result",
        state_bundle,
        est_progress_percentage,
      },
    });
  }
  fail(message = "boom") {
    this.onerror?.({ message });
  }
  // what the worker posts when an op throws
  crash(message = "panic") {
    this.onmessage?.({ data: { type: "error", message } });
  }
}

export function install_fake_worker() {
  FakeWorker.all = [];
  vi.stubGlobal("Worker", FakeWorker);
}

// workers not yet terminated
export const live = () => FakeWorker.all.filter((w) => !w.terminated);
