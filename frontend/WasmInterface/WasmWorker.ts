import init, {
  optimize_average_wrapper,
  histogram_wrapper,
} from "@/../crates/wasm/pkg/hf_wasm.js";
import { Payload } from "./PayloadBuilder";
import { Upgrade } from "@/Utils/KeyedUpgrades";

export enum WasmOp {
  // EvaluateAverage,
  OptimizeAverage,
  Histogram,
  // Parser,
}
// THESE BELOW DIRECTLY CORRESPOND TO A RUST STRUCT

export type HistogramPair = [number, number];
// export interface HistogramOutputs {
//     cum_percentiles: HistogramPair[][]
//     average: number[]
//     state_bundle: StateBundle
//     bound_chance: number[]
//     tradable_chance: number[]
// }
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

// async function ParserWasm(payload: Payload): Promise<StateBundle> {
//     await init()
//     return parser_wrapper(payload)
// }
async function OptimizeAverageWasm(payload: Payload): Promise<StateBundle> {
  await init();
  return optimize_average_wrapper(payload);
}
// async function EvaluateAverageWasm(payload: Payload): Promise<StateBundle> {
//     await init()
//     return evaluate_average_wrapper(payload)
// }
async function HistogramWasm(payload: Payload): Promise<HistogramOutputs> {
  await init();
  return histogram_wrapper(payload);
}
self.addEventListener("message", async (ev) => {
  const msg = ev.data;
  const start_time = performance.now();

  const { payload, wasm_op } = msg;

  console.log(WasmOp[wasm_op], "Began", payload);
  let result;

  // if (wasm_op == WasmOp.EvaluateAverage) {
  //     result = await EvaluateAverageWasm(payload)
  // } else
  if (wasm_op == WasmOp.OptimizeAverage) {
    result = await OptimizeAverageWasm(payload);
  } else if (wasm_op == WasmOp.Histogram) {
    result = await HistogramWasm(payload);
  } else //     if (wasm_op == WasmOp.Parser) {
  //     result = await ParserWasm(payload)
  // } else
  {
    return; // react dev tool shenanigans
  }
  console.log(
    WasmOp[wasm_op],
    "done",
    ((performance.now() - start_time) / 1000).toFixed(4),
    result,
  );
  // console.log(WasmOp[wasm_op] + " finished after " + String(((Date.now() - start_time) / 1000).toFixed(2)) + "s")
  self.postMessage({ type: "result", result });
});
