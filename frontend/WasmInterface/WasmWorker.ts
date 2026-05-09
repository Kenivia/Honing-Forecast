import init, { optimize_average_wrapper, histogram_wrapper } from "@/../crates/wasm/pkg/hf_wasm.js"
import { Payload } from "./PayloadBuilder"
import { HistogramOutputs, StateBundle, WasmOp } from "@/Utils/Interfaces"

// async function ParserWasm(payload: Payload): Promise<StateBundle> {
//     await init()
//     return parser_wrapper(payload)
// }
async function OptimizeAverageWasm(payload: Payload): Promise<StateBundle> {
    await init()
    return optimize_average_wrapper(payload)
}
// async function EvaluateAverageWasm(payload: Payload): Promise<StateBundle> {
//     await init()
//     return evaluate_average_wrapper(payload)
// }
async function HistogramWasm(payload: Payload): Promise<HistogramOutputs> {
    await init()
    return histogram_wrapper(payload)
}
self.addEventListener("message", async (ev) => {
    const msg = ev.data
    // let start_time = Date.now()

    const { payload, wasm_op } = msg

    console.log(WasmOp[wasm_op], "Began", payload)
    let result

    // if (wasm_op == WasmOp.EvaluateAverage) {
    //     result = await EvaluateAverageWasm(payload)
    // } else
    if (wasm_op == WasmOp.OptimizeAverage) {
        result = await OptimizeAverageWasm(payload)
    } else if (wasm_op == WasmOp.Histogram) {
        result = await HistogramWasm(payload)
    } else //     if (wasm_op == WasmOp.Parser) {
    //     result = await ParserWasm(payload)
    // } else
    {
        return // react dev tool shenanigans
    }
    console.log(WasmOp[wasm_op], "done", result)
    // console.log(WasmOp[wasm_op] + " finished after " + String(((Date.now() - start_time) / 1000).toFixed(2)) + "s")
    self.postMessage({ type: "result", result })
})
