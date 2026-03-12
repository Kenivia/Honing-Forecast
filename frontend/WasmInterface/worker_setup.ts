import { ref, onUnmounted } from "vue"
import { WasmOp } from "./js_to_wasm"
import { buildPayload } from "./payload"
const createWorker = () => new Worker(new URL("./js_to_wasm.ts", import.meta.url), { type: "module" })

export function createWorkerBundle() {
    let worker = null
    const status = ref("idle")
    const result = ref(null)
    const error = ref(null)
    let debounceTimer = null

    function _launch(wasm_op: WasmOp) {
        cancel()

        status.value = "running"
        result.value = null
        error.value = null

        worker = createWorker()

        worker.onmessage = (e) => {
            result.value = e.result
            if (e.type === "result") {
                status.value = "done"
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
            status.value = "debouncing"
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
            status.value = "cancelled"
        }
    }

    onUnmounted(cancel)

    return { status, result, error, start, cancel }
}
