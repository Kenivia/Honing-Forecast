import init, { evaluate_average_wrapper, optimize_average_wrapper, histogram_wrapper, parser_wrapper } from "@/../crates/wasm/pkg/hf_wasm.js"
import { EvalPayload } from "./payload"
import { HistogramOutputs, StateBundle } from "@/Utils/Interfaces"

const LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver"]

export enum WasmOp {
    EvaluateAverage,
    OptimizeAverage,
    Histogram,
    Parser,
}
async function ParserWasm(payload: EvalPayload): Promise<StateBundle> {
    await init()
    return parser_wrapper(payload)
}
async function OptimizeAverageWasm(payload: EvalPayload): Promise<StateBundle> {
    await init()
    return optimize_average_wrapper(payload)
}
async function EvaluateAverageWasm(payload: EvalPayload): Promise<StateBundle> {
    await init()
    return evaluate_average_wrapper(payload)
}
async function HistogramWasm(payload: EvalPayload): Promise<HistogramOutputs> {
    await init()
    return histogram_wrapper(payload)
}

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

    console.log(WasmOp[wasm_op] + " finished after " + String(((Date.now() - start_time) / 1000).toFixed(2)) + "s")
    self.postMessage({ type: "result", id, result })
})
