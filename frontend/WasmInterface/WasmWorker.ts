import init, {
  optimize_average_wrapper,
  histogram_wrapper,
  marginal_value_wrapper,
  cropper_wrapper,
  reserve_buffer_wrapper,
  take_timings,
  replay_wrapper,
  ocr_init_wrapper,
  ocr_wrapper,
  recorder_buffer,
  record_frame,
} from "@/../crates/wasm/pkg/hf_wasm.js";
import type { Payload } from "./PayloadBuilder";
import { ocr_job_buffers, WasmOp } from "./WasmTypes";
import type {
  OcrJob,
  OneIconConfig,
} from "@/Components/Character/InventoryScanner/LoadStorage";

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
// a frame's pixels as the scanner worker forwarded them
function record({ pixels }, memory: WebAssembly.Memory, timings) {
  const pointer = recorder_buffer(pixels.length);
  const copy_start = performance.now();
  new Uint8Array(memory.buffer, pointer, pixels.length).set(pixels);
  timings.copy = performance.now() - copy_start;
  const record_start = performance.now();
  const out = record_frame();
  timings.record = performance.now() - record_start;
  take_timings();
  return out;
}

// where frames are copied to; the scanner state itself stays inside wasm
let scanner_buffer: { pointer: number; size: number } | null = null;

// One instance per worker: a message that comes in while the first is still loading it (the
// recorder gets frames back to back) must wait for that one, not load another.
let wasm_ready: ReturnType<typeof init> | null = null;

// a throw, a wasm panic included, is reported rather than left as an unhandled rejection
self.addEventListener("message", async (ev) => {
  try {
    await handle(ev.data);
  } catch (err) {
    console.error(err);
    self.postMessage({ type: "error", message: String(err) });
  }
});

async function handle(msg) {
  const start_time = performance.now();
  // scan timings in ms, see Stream.vue
  const timings: Record<string, any> = { to_worker: abs_now() - msg.posted_at };

  const { payload, wasm_op } = msg;

  let result;

  const wasm = await (wasm_ready ??= init());
  timings.init = performance.now() - start_time;
  // never log scanner state, Firefox keeps logged objects alive even with devtools closed
  const loggable = wasm_op <= WasmOp.MarginalValue;
  if (loggable) console.log(WasmOp[wasm_op], "Began", payload);

  if (wasm_op == WasmOp.OptimizeAverage) {
    result = await optimize_average_wrapper(payload);
  } else if (wasm_op == WasmOp.Histogram) {
    result = to_records(await histogram_wrapper(payload));
  } else if (wasm_op == WasmOp.MarginalValue) {
    result = Object.fromEntries(await marginal_value_wrapper(payload));
  } else if (wasm_op == WasmOp.Cropper) {
    // the main thread reads the frame and transfers it with the op
    const { frame, replay, forward, ...options } = payload;

    if (replay) {
      // a capture's input record stands in for the frame
      const call_start = performance.now();
      result = replay_wrapper(replay, options);
      timings.wasm_call = performance.now() - call_start;
      timings.rust = take_timings();
    } else
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
        // the recorder gets the pixels from here rather than copying the frame again
        let pixels: Uint8Array | null = null;
        if (forward) {
          const forward_start = performance.now();
          pixels = dest.slice(0, requiredSize);
          timings.forward = performance.now() - forward_start;
        }

        const call_start = performance.now();
        result = await cropper_wrapper(options);
        timings.wasm_call = performance.now() - call_start;
        timings.rust = take_timings();
        if (pixels) result.pixels = pixels;
      } catch (err) {
        console.error("Error processing frame:", err);
      } finally {
        frame.close();
      }
  } else if (wasm_op == WasmOp.Reserve) {
    result = await reserve_buffer_wrapper(payload);
    scanner_buffer = result.buffer;
  } else if (wasm_op == WasmOp.Record) {
    result = record(payload, wasm.memory, timings);
  } else if (wasm_op == WasmOp.OcrInit) {
    ocr_init_wrapper(payload.model, payload.config);
  } else if (wasm_op == WasmOp.Ocr) {
    // a batch of text lines from the scanner worker, relayed by the main thread
    result = payload.map((job) => [job.id, ocr_wrapper(job)]);
    take_timings();
  } else {
    return; // not an op
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
  // a scan's text crops and slot images are moved, not copied
  const transfer: ArrayBuffer[] =
    wasm_op == WasmOp.Cropper && result
      ? [
          ...result.ocr_jobs.flatMap(ocr_job_buffers),
          ...(result.pixels ? [result.pixels.buffer] : []),
          ...result.slots.flatMap((slot) =>
            slot.image ? [slot.image.data.buffer] : [],
          ),
        ]
      : wasm_op == WasmOp.Record
        ? [result.buffer]
        : [];
  self.postMessage({ type: "result", result, timings }, { transfer });
}

// comparable between the main thread and the worker
function abs_now() {
  return performance.timeOrigin + performance.now();
}
