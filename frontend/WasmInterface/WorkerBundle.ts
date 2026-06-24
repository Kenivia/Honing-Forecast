import { ref, Ref, shallowRef } from "vue";
import { Payload } from "./PayloadBuilder";
import { ScannerState, WasmOp } from "./WasmWorker";

const createWorker = () =>
  new Worker(new URL("./WasmWorker.ts", import.meta.url), { type: "module" });

export function create_worker_bundle() {
  let worker = null;
  const status: Ref<"idle" | "busy"> = ref("idle");
  const error = ref(null);
  const result = shallowRef(null);
  const est_progress_percentage = ref(0);
  const last_intermediate_time = ref(0);
  const run_counter = ref(0);

  let debounceTimer = null;
  let throttle_timer: ReturnType<typeof setTimeout> | null = null;
  let throttle_pending: {
    wasm_op: WasmOp;
    payload: any;
  } | null = null;
  let throttle_ready = true;

  function _try_flush_throttle() {
    // console.log("try", throttle_pending, throttle_ready, status.value)
    if (throttle_pending === null) return;
    if (status.value === "busy") return;
    if (!throttle_ready) return;

    const { wasm_op, payload } = throttle_pending;
    throttle_pending = null;
    throttle_ready = false;

    throttle_timer = setTimeout(() => {
      throttle_ready = true;
      _try_flush_throttle();
    }, 200);

    _launch(wasm_op, payload, false, () => {
      // if (throttle_pending) {  this check's unnecessary cos the inside checks anyway

      // callback should go here if i decide to add

      _try_flush_throttle();

      // }
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
    callback?: (result) => void,
  ) {
    if (cancel) {
      cancel_worker();
    }
    run_counter.value += 1;

    status.value = "busy";
    error.value = null;
    est_progress_percentage.value = 0;
    if (!worker) {
      worker = createWorker();
    }

    worker.onmessage = (e) => {
      // console.log(e)

      if (e.data.type === "result") {
        result.value = e.data.result;
        // console.log(result.value);
        status.value = "busy";
        est_progress_percentage.value = 100;
        // console.log(mapToObject(toRaw(result.value)?.adv_cache) ?? null)
        if (cancel) {
          worker.terminate();
          worker = null;
        }
        if (callback) {
          callback(result.value);
        }
      } else {
        // 1 sec interval from rust's side
        if (e.data.state_bundle) {
          // result.value = e.data.state_bundle;
          last_intermediate_time.value = performance.now();

          if (callback) {
            // console.log("callback");
            callback(e.data.state_bundle);
          }
        }

        // console.log(
        //   e.data.state_bundle,
        //   e.data.est_progress_percentage,
        //   !!e.data.state_bundle,
        //   last_intermediate_time.value - performance.now(),
        // );
        est_progress_percentage.value = e.data.est_progress_percentage;
      }
    };

    worker.onerror = (e) => {
      error.value = e.message;
      status.value = "idle";
      worker = null;
    };

    console.log(WasmOp[wasm_op], payload);
    // console.log(JSON.parse(JSON.stringify(toRaw(buildPayload(wasm_op)))))
    worker.postMessage(
      { type: "message", wasm_op, payload },
      { transfer: payload.readable ? [payload.readable] : [] },
    );
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
    status.value = "idle";
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
    result.value = null;
    est_progress_percentage.value = 0;
  }

  // this is not necessary because it's done explicitly in charview
  // onUnmounted(cancel);

  return {
    status,
    result,
    error,
    est_progress_percentage,
    debounced_start,
    throttled_start,
    cancel,
    cancel_and_clear_prev_result,
    run_counter,
  };
}
