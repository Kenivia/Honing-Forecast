import { shallowReactive } from "vue";
import { ocr_job_buffers, WasmOp } from "./WasmWorker";

const createWorker = () =>
  new Worker(new URL("./WasmWorker.ts", import.meta.url), { type: "module" });

export interface WorkerBundle {
  status: "idle" | "busy";
  result: any;
  error: string | null;
  est_progress_percentage: number;
  run_counter: number;
  debounced_start: (
    wasm_op: WasmOp,
    payload: any,
    callback?: (result: any, timings?: any) => void,
    debounce?: number,
    cancel?: boolean,
  ) => void;
  throttled_start: (wasm_op: WasmOp, payload: any) => void;
  cancel: () => void;
  cancel_and_clear_prev_result: () => void;
}

// shallowReactive, not refs: the fields read as plain values whether or not the bundle
// sits inside a store, and `result` (a large wasm payload, with Maps in it) is never
// deep-wrapped.
export function create_worker_bundle(): WorkerBundle {
  let worker: Worker | null = null;
  let debounceTimer = null;
  let throttle_timer: ReturnType<typeof setTimeout> | null = null;
  let throttle_pending: {
    wasm_op: WasmOp;
    payload: any;
  } | null = null;
  let throttle_ready = true;

  const bundle: WorkerBundle = shallowReactive({
    status: "idle",
    result: null,
    error: null,
    est_progress_percentage: 0,
    run_counter: 0,
    debounced_start,
    throttled_start,
    cancel,
    cancel_and_clear_prev_result,
  });

  function _try_flush_throttle() {
    if (throttle_pending === null) return;
    if (bundle.status === "busy") return;
    if (!throttle_ready) return;

    const { wasm_op, payload } = throttle_pending;
    throttle_pending = null;
    throttle_ready = false;

    throttle_timer = setTimeout(() => {
      throttle_ready = true;
      _try_flush_throttle();
    }, 200);

    _launch(wasm_op, payload, false, () => {
      _try_flush_throttle();
    });
  }

  function throttled_start(wasm_op: WasmOp, payload: any) {
    throttle_pending = { wasm_op, payload };
    _try_flush_throttle();
  }

  function _launch(
    wasm_op: WasmOp,
    payload: any,
    cancel: boolean,
    callback?: (result, timings?) => void,
  ) {
    if (cancel) {
      cancel_worker();
    }
    const abs_now = () => performance.timeOrigin + performance.now();
    let post_ms = 0;
    bundle.run_counter += 1;

    bundle.status = "busy";
    bundle.error = null;
    bundle.est_progress_percentage = 0;
    if (!worker) {
      worker = createWorker();
    }

    worker.onmessage = (e) => {
      if (e.data.type === "result") {
        const timings = e.data.timings;
        timings.to_main = abs_now() - timings.posted_at;
        timings.post = post_ms;
        bundle.result = e.data.result;
        bundle.status = "idle";
        bundle.est_progress_percentage = 100;
        if (cancel) {
          worker.terminate();
          worker = null;
        }
        if (callback) {
          callback(bundle.result, timings);
        }
      } else {
        // 1 sec interval from rust's side
        if (e.data.state_bundle) {
          if (callback) {
            callback(e.data.state_bundle);
          }
        }

        bundle.est_progress_percentage = e.data.est_progress_percentage;
      }
    };

    worker.onerror = (e) => {
      bundle.error = e.message;
      bundle.status = "idle";
      worker = null;
    };

    // scanner ops run many times a second and are not logged, see WasmWorker
    if (wasm_op <= WasmOp.Histogram) console.log(WasmOp[wasm_op], payload);
    const post_start = performance.now();
    worker.postMessage(
      { type: "message", wasm_op, payload, posted_at: abs_now() },
      // a transferred frame leaves no copy behind on this thread, nor do a batch's crops
      {
        transfer: payload?.frame
          ? [payload.frame]
          : wasm_op == WasmOp.Ocr
            ? payload.flatMap(ocr_job_buffers)
            : [],
      },
    );
    post_ms = performance.now() - post_start;
  }

  function debounced_start(
    wasm_op: WasmOp,
    payload: any,
    callback?: (result) => void,
    debounce = 200,
    cancel = true,
  ) {
    if (debounce > 0) {
      clearTimeout(debounceTimer);

      debounceTimer = setTimeout(
        () => _launch(wasm_op, payload, cancel, callback),
        debounce,
      );
    } else {
      _launch(wasm_op, payload, cancel, callback);
    }
  }

  function cancel_worker() {
    if (worker) {
      worker.terminate();
      worker = null;
    }
    bundle.status = "idle";
  }

  function cancel() {
    clearTimeout(debounceTimer);
    clearTimeout(throttle_timer);
    debounceTimer = null;
    throttle_timer = null;
    throttle_pending = null;
    throttle_ready = true;
    cancel_worker();
  }

  function cancel_and_clear_prev_result() {
    cancel();
    bundle.result = null;
    bundle.est_progress_percentage = 0;
  }

  // cancelling on unmount is done explicitly in CharView
  return bundle;
}
