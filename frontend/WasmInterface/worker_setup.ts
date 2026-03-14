import { ref, onUnmounted, Ref, toRaw } from "vue"
import { WasmOp } from "./js_to_wasm"
import { HistogramOutputs, StateBundle } from "@/Utils/Interfaces"
import { buildPayload } from "./payload"
const createWorker = () => new Worker(new URL("./js_to_wasm.ts", import.meta.url), { type: "module" })

export function createWorkerBundle(inp: Ref<null | StateBundle | HistogramOutputs>) {
    let worker = null
    const status: Ref<"idle" | "success" | "busy" | "error"> = ref("idle")
    const error = ref(null)
    const result = inp
    let debounceTimer = null

    function _launch(wasm_op: WasmOp) {
        cancel()

        status.value = "busy"
        error.value = null

        worker = createWorker()

        worker.onmessage = (e) => {
            // console.log(e)

            if (e.data.type === "result") {
                result.value = e.data.result
                status.value = "success"
                worker.terminate()
                worker = null
            } else {
                result.value = e.data.state_bundle
            }
        }

        worker.onerror = (e) => {
            error.value = e.message
            status.value = "error"
            worker = null
        }

        // console.log(WasmOp[wasm_op], toRaw(buildPayload(wasm_op)))
        // console.log(JSON.parse(JSON.stringify(toRaw(buildPayload(wasm_op)))))
        worker.postMessage({ type: "message", wasm_op, payload: JSON.parse(JSON.stringify(toRaw(buildPayload(wasm_op)))) })
    }

    function start(wasm_op: WasmOp, debounce?: number) {
        if (debounce > 0) {
            clearTimeout(debounceTimer)
            status.value = "busy"
            debounceTimer = setTimeout(() => _launch(wasm_op), debounce)
        } else {
            _launch(wasm_op)
        }
    }

    function cancel() {
        clearTimeout(debounceTimer)
        debounceTimer = null
        if (worker) {
            worker.terminate()
            worker = null
            status.value = "idle"
        }
    }

    onUnmounted(cancel)

    return { status, result, error, start, cancel }
}
