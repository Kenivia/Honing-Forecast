import init, {
  optimize_average_wrapper,
  histogram_wrapper,
  setup_wrapper,
  cropper_wrapper,
  reserve_buffer_wrapper,
  take_timings,
  ocr_init_wrapper,
  ocr_wrapper,
} from "@/../crates/wasm/pkg/hf_wasm.js";
import { Payload } from "./PayloadBuilder";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import { OneIconConfig } from "@/Components/Character/InventoryScanner/LoadStorage";

export enum WasmOp {
  OptimizeAverage,
  Histogram,
  Setup,
  Cropper,
  Reserve,
  OcrInit,
  Ocr,
}

// THESE BELOW DIRECTLY CORRESPOND TO A RUST STRUCT
export type HistogramPair = [number, number];
// Rust's label keyed maps, flattened to records by to_records below.
export type ByLabel<T> = Record<string, T>;
export interface HistogramOutputs {
  cum_percentiles: ByLabel<HistogramPair[]>;

  // chance the first k ownership bands cover every upgrade, as [bound, +roster, +tradable]
  chance_within: ByLabel<[number, number, number]>;
  avg_used: ByLabel<number>;

  // one entry per plan in the payload, in payload order
  gold_per_plan: ByLabel<number>[];
  metric_per_plan: number[];

  juice_info: any;
}

// Rust hash maps arrive as JS Maps. Every label keyed field becomes a plain record here,
// so nothing downstream has to know which is which.
const KEYED_FIELDS = ["cum_percentiles", "chance_within", "avg_used"];
function to_records(result: any): any {
  for (const field of KEYED_FIELDS) {
    if (result[field] instanceof Map) {
      result[field] = Object.fromEntries(result[field]);
    }
  }
  if (Array.isArray(result.gold_per_plan)) {
    result.gold_per_plan = result.gold_per_plan.map((x) =>
      x instanceof Map ? Object.fromEntries(x) : x,
    );
  }
  return result;
}
export interface StateBundle {
  upgrade_arr: Upgrade[];
  special_state: number[];
  special_invalid_index?: number;
  latest_special_probs?: number[];
  min_resolution: number;
  gold_breakdown?: number[];
  prep_output: any;
  special_cache: any;
  adv_cache: any;
  metric?: number;
}

// where frames are copied to; the scanner state itself stays inside wasm
let scanner_buffer: { pointer: number; size: number } | null = null;

self.addEventListener("message", async (ev) => {
  const msg = ev.data;
  const start_time = performance.now();
  // scan timings in ms, see Stream.vue
  const timings: Record<string, any> = { to_worker: abs_now() - msg.posted_at };

  const { payload, wasm_op } = msg;

  let result;

  const wasm = await init();
  timings.init = performance.now() - start_time;
  // never log scanner state, Firefox keeps logged objects alive even with devtools closed
  const loggable = wasm_op <= WasmOp.Histogram;
  if (loggable) console.log(WasmOp[wasm_op], "Began", payload);

  if (wasm_op == WasmOp.OptimizeAverage) {
    result = await optimize_average_wrapper(payload);
  } else if (wasm_op == WasmOp.Histogram) {
    result = to_records(await histogram_wrapper(payload));
  } else if (wasm_op == WasmOp.Cropper || wasm_op == WasmOp.Setup) {
    // the main thread reads the frame and transfers it with the op
    const { frame, ...options } = payload;

    try {
      const requiredSize = frame.allocationSize({ format: "RGBA" });
      if (scanner_buffer.size < requiredSize) {
        throw new Error(
          `buffer too small, need ${requiredSize}, got ${scanner_buffer.size}`,
        );
      }

      const dest = new Uint8Array(
        wasm.memory.buffer,
        scanner_buffer.pointer,
        scanner_buffer.size,
      );

      const copy_start = performance.now();
      await frame.copyTo(dest, { format: "RGBA" });
      frame.close();
      timings.copy = performance.now() - copy_start;

      if (wasm_op == WasmOp.Cropper) {
        const call_start = performance.now();
        result = await cropper_wrapper(options);
        timings.wasm_call = performance.now() - call_start;
        timings.rust = take_timings();
      } else {
        result = await setup_wrapper(options);
      }
    } catch (err) {
      console.error("Error processing frame:", err);
    } finally {
      frame.close();
    }
  } else if (wasm_op == WasmOp.Reserve) {
    result = await reserve_buffer_wrapper(payload);
    scanner_buffer = result.buffer;
  } else if (wasm_op == WasmOp.OcrInit) {
    ocr_init_wrapper(payload);
  } else if (wasm_op == WasmOp.Ocr) {
    // a batch of text lines from the scanner worker, relayed by the main thread
    result = payload.map((job) => [
      job.id,
      ocr_wrapper(job.width, job.height, job.data, job.numbers),
    ]);
    take_timings();
  } else {
    return; // react dev tool shenanigans
  }
  if (loggable) {
    console.log(
      WasmOp[wasm_op],
      "done",
      (performance.now() - start_time).toFixed(0),
      result,
    );
  }

  timings.worker = performance.now() - start_time;
  timings.posted_at = abs_now();
  self.postMessage({ type: "result", result, timings });
});

// comparable between the main thread and the worker
function abs_now() {
  return performance.timeOrigin + performance.now();
}
