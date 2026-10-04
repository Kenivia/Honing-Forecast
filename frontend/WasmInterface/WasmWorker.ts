import init, {
  optimize_average_wrapper,
  histogram_wrapper,
  setup_wrapper,
  cropper_wrapper,
  reserve_buffer_wrapper,
  dealloc_buffer_wrapper,
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

self.addEventListener("message", async (ev) => {
  const msg = ev.data;
  const start_time = performance.now();

  const { payload, wasm_op } = msg;

  let result;

  const wasm = await init();
  // never log scanner state, Firefox keeps logged objects alive even with devtools closed
  const loggable = wasm_op <= WasmOp.Histogram;
  console.log(WasmOp[wasm_op], "Began", loggable ? payload : "");

  if (wasm_op == WasmOp.OptimizeAverage) {
    result = await optimize_average_wrapper(payload);
  } else if (wasm_op == WasmOp.Histogram) {
    result = await histogram_wrapper(payload);
  } else if (wasm_op == WasmOp.Cropper || wasm_op == WasmOp.Setup) {
    // the main thread reads the frame and transfers it with the op
    const { scanner_state, frame } = payload;

    try {
      const requiredSize = frame.allocationSize({ format: "RGBA" });
      if (scanner_state.buffer.size < requiredSize) {
        throw new Error(
          `buffer too small, need ${requiredSize}, got ${scanner_state.buffer.size}`,
        );
      }

      const dest = new Uint8Array(
        wasm.memory.buffer,
        scanner_state.buffer.pointer,
        scanner_state.buffer.size,
      );

      await frame.copyTo(dest, { format: "RGBA" });
      frame.close();
      console.log("transfer done", (performance.now() - start_time).toFixed(0));

      if (wasm_op == WasmOp.Cropper) {
        result = await cropper_wrapper(scanner_state);
      } else {
        result = await setup_wrapper(scanner_state);
      }
    } catch (err) {
      console.error("Error processing frame:", err);
    } finally {
      frame.close();
    }
  } else if (wasm_op == WasmOp.Reserve) {
    result = await reserve_buffer_wrapper(payload);
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
    loggable ? result : "",
  );

  self.postMessage({ type: "result", result });
});
