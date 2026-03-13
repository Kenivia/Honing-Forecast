import { ref, onUnmounted, Ref } from "vue"
import { WasmOp } from "./js_to_wasm"
import { EvalPayload } from "./payload"
import { StateBundle } from "@/Utils/Interfaces"
const createWorker = () => new Worker(new URL("./js_to_wasm.ts", import.meta.url), { type: "module" })

export function createWorkerBundle(resultRef?: Ref<null | StateBundle>) {
    let worker = null
    const status: Ref<"idle" | "success" | "busy" | "error"> = ref("idle")
    const error = ref(null)
    const result = resultRef ?? ref(null)
    let debounceTimer = null

    function _launch(wasm_op: WasmOp, payload: EvalPayload) {
        cancel()

        status.value = "busy"
        result.value = null
        error.value = null

        worker = createWorker()

        worker.onmessage = (e) => {
            result.value = e.result
            if (e.type === "result") {
                status.value = "success"
                worker = null
                worker.terminate()
            }
        }

        worker.onerror = (e) => {
            error.value = e.message
            status.value = "error"
            worker = null
        }

        worker.postMessage({ type: "message", wasm_op, payload })
    }

    function start(wasm_op: WasmOp, payload: EvalPayload, debounce?: number) {
        if (debounce > 0) {
            clearTimeout(debounceTimer)
            status.value = "busy"
            debounceTimer = setTimeout(() => _launch(wasm_op, payload), debounce)
        } else {
            _launch(wasm_op, payload)
        }
    }

    function cancel() {
        clearTimeout(debounceTimer)
        debounceTimer = null
        if (worker) {
            worker = null
            worker.terminate()
            status.value = "idle"
        }
    }

    onUnmounted(cancel)

    return { status, result: result, error, start, cancel }
}
