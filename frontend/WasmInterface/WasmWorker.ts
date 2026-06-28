import init, {
  optimize_average_wrapper,
  histogram_wrapper,
  cropper_wrapper,
  reserve_buffer_wrapper,
  dealloc_buffer_wrapper,
} from "@/../crates/wasm/pkg/hf_wasm.js";
import { Payload } from "./PayloadBuilder";
import { Upgrade } from "@/Utils/KeyedUpgrades";

export enum WasmOp {
  OptimizeAverage,
  Histogram,
  Cropper,
  Reserve,
  Dealloc,
}

// THESE BELOW DIRECTLY CORRESPOND TO A RUST STRUCT
export type HistogramPair = [number, number];
export interface HistogramOutputs {
  cum_percentiles: HistogramPair[][];

  chances_arr: number[][]; //  [treatment plan][material type] for all 3 of these
  gold_breakdown_arr: number[][];
  metrics_arr: number[];

  avg_breakdown: number[];

  juice_info: any;
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

export interface Buffer {
  pointer: number;
  size: number;
}
export interface ScannerState {
  slot_infos: any;
  anchors: any;
  screen_info: any;
  pending_jobs: any;

  buffer: Buffer;
}

let reader;

self.addEventListener("message", async (ev) => {
  const msg = ev.data;
  const start_time = performance.now();

  const { payload, wasm_op } = msg;

  let result;

  const wasm = await init();
  console.log(WasmOp[wasm_op], "Began", payload);

  if (wasm_op == WasmOp.OptimizeAverage) {
    result = await optimize_average_wrapper(payload);
  } else if (wasm_op == WasmOp.Histogram) {
    result = await histogram_wrapper(payload);
  } else if (wasm_op == WasmOp.Cropper) {
    const { value: frame, done } = await reader.read();
    // console.log("frame arrived ", (performance.now() - start_time).toFixed(0));
    // console.log("read", frame, done);
    if (done) {
      console.log("done");
      result = payload;
    } else {
      // console.log(wasm.memory.buffer);
      const dest = new Uint8Array(
        wasm.memory.buffer,
        payload.buffer.pointer,
        payload.buffer.size,
      );

      await frame.copyTo(dest, { format: "RGBA" });
      frame.close();
      // console.log(
      //   "transfer ",
      //   "done",
      //   (performance.now() - start_time).toFixed(0),
      // );
      result = await cropper_wrapper(payload);
    }

    //
  } else if (wasm_op == WasmOp.Reserve) {
    reader = payload.readable.getReader();
    result = await reserve_buffer_wrapper(payload.scanner_state);
    // console.log("post reserve", wasm.memory.buffer);
  } else if (wasm_op == WasmOp.Dealloc) {
    result = await dealloc_buffer_wrapper(payload);
  } else {
    return; // react dev tool shenanigans
  }
  console.log(
    WasmOp[wasm_op],
    "done",
    (performance.now() - start_time).toFixed(0),
    result,
  );

  self.postMessage({ type: "result", result });
});
