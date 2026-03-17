import { ref, onUnmounted, Ref, toRaw } from "vue"
import { WasmOp } from "./js_to_wasm"
import { HistogramOutputs, StateBundle } from "@/Utils/Interfaces"
import { buildPayload } from "./payload"
import { mapToObject } from "@/Utils/Helpers"
import equal from "fast-deep-equal"

const createWorker = () => new Worker(new URL("./js_to_wasm.ts", import.meta.url), { type: "module" })

export function createWorkerBundle() {
    let worker = null
    const status: Ref<"idle" | "success" | "busy" | "error"> = ref("idle")
    const error = ref(null)
    const result = ref(null)
    const est_progress_percentage = ref(0)
    const last_payload = ref(null)
    let debounceTimer = null

    function _launch(wasm_op: WasmOp, callback?: (result) => void) {
        cancel()

        status.value = "busy"
        error.value = null
        est_progress_percentage.value = 0
        worker = createWorker()

        worker.onmessage = (e) => {
            // console.log(e)

            if (e.data.type === "result") {
                result.value = e.data.result
                status.value = "success"
                est_progress_percentage.value = 100
                // console.log(mapToObject(toRaw(result.value)?.adv_cache) ?? null)
                worker.terminate()
                worker = null
            } else {
                result.value = e.data.state_bundle
                est_progress_percentage.value = e.data.est_progress_percentage
            }
            if (callback) {
                callback(result.value)
            }
        }

        worker.onerror = (e) => {
            error.value = e.message
            status.value = "error"
            worker = null
            last_payload.value = null
        }
        let payload = structuredClone(toRaw(buildPayload(wasm_op)))
        if (last_payload.value == null || !equal(payload, structuredClone(toRaw(last_payload.value)))) {
            console.log(WasmOp[wasm_op], payload, structuredClone(toRaw(last_payload.value)))
            worker.postMessage({ type: "message", wasm_op, payload })
            last_payload.value = payload
        }
    }

    function start(wasm_op: WasmOp, callback?: (result) => void, debounce?: number) {
        if (debounce > 0) {
            clearTimeout(debounceTimer)
            status.value = "busy"
            debounceTimer = setTimeout(() => _launch(wasm_op), debounce)
        } else {
            _launch(wasm_op, callback)
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
    function cancel_and_clear_prev_result() {
        cancel()
        result.value = null
        est_progress_percentage.value = 0
    }

    onUnmounted(cancel)

    return { status, result, error, est_progress_percentage, start, cancel, cancel_and_clear_prev_result }
}
