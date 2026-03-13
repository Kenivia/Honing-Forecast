import init, {
    // initThreadPool,
    evaluate_average_wrapper,
    optimize_average_wrapper,
    histogram_wrapper,
    parser_wrapper,

    // cost_to_chance_arr_wrapper,
    // parser_wrapper_unified,
    // average_cost_wrapper,
    // monte_carlo_wrapper,
} from "@/../crates/wasm/pkg/hf_wasm.js"
import { EvalPayload } from "./payload"
import { StateBundle } from "@/Utils/Interfaces"

const LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver"]

export enum WasmOp {
    EvaluateAverage,
    OptimizeAverage,
    Histogram,
    Parser,
}
async function ParserWasm(payload: EvalPayload) {
    await init()
    return parser_wrapper(payload)
}
async function OptimizeAverageWasm(state_bundle: StateBundle) {
    await init()
    return optimize_average_wrapper(state_bundle)
}
async function EvaluateAverageWasm(state_bundle: StateBundle) {
    await init()
    return evaluate_average_wrapper(state_bundle)
}
async function HistogramWasm(state_bundle: StateBundle) {
    await init()
    return histogram_wrapper(state_bundle)
}

// async function CostToChanceArrWasm(payload: any) {
//     await init()
//     return (cost_to_chance_arr_wrapper as any)(payload)
// }

// async function ParserWasmUnified(payload: any) {
//     await init()
//     return (parser_wrapper_unified as any)(payload)
// }

// async function AverageCostWasm(payload: any) {
//     await init()
//     return (average_cost_wrapper as any)(payload)
// }

self.addEventListener("message", async (ev) => {
    const msg = ev.data
    let start_time = Date.now()

    const { id, payload, wasm_op } = msg

    console.log(WasmOp[wasm_op], "Began")
    console.log(payload)
    let result

    if (wasm_op == WasmOp.EvaluateAverage) {
        result = await EvaluateAverageWasm(payload)
    } else if (wasm_op == WasmOp.OptimizeAverage) {
        result = await OptimizeAverageWasm(payload)
    } else if (wasm_op == WasmOp.Histogram) {
        result = await HistogramWasm(payload)
    } else if (wasm_op == WasmOp.Parser) {
        result = await ParserWasm(payload)
    }
    // result.run_time = ((Date.now() - start_time) / 1000).toFixed(2)
    console.log(WasmOp[wasm_op] + " finished after " + String(((Date.now() - start_time) / 1000).toFixed(2)) + "s")
    self.postMessage({ type: "result", id, result })
})
