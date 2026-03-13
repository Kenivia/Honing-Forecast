import { ref, onUnmounted, Ref } from "vue"
import { WasmOp } from "./js_to_wasm"
import { StateBundle } from "@/Utils/Interfaces"
import { buildPayload } from "./payload"
const createWorker = () => new Worker(new URL("./js_to_wasm.ts", import.meta.url), { type: "module" })

export function createWorkerBundle(given_result?: null | StateBundle) {
    let worker = null
    const status: Ref<"idle" | "success" | "busy" | "error"> = ref("idle")
    const error = ref(null)
    const result = ref(given_result)
    let debounceTimer = null

    function _launch(wasm_op: WasmOp) {
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

        worker.postMessage({ type: "message", wasm_op, payload: buildPayload(wasm_op) })
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
            worker = null
            worker.terminate()
            status.value = "idle"
        }
    }

    onUnmounted(cancel)

    return { status, result: result, error, start, cancel }
}
