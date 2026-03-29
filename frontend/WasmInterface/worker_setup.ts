import { ref, onUnmounted, Ref, shallowRef } from "vue"
import { EvalPayload } from "./payload"
import { WasmOp } from "@/Utils/Interfaces"

const createWorker = () => new Worker(new URL("./js_to_wasm.ts", import.meta.url), { type: "module" })

export function createWorkerBundle() {
    let worker = null
    const status: Ref<"idle" | "success" | "busy" | "error"> = ref("idle")
    const error = ref(null)
    const result = shallowRef(null)
    const est_progress_percentage = ref(0)
    const last_intermediate_time = ref(0)

    let debounceTimer = null
    let throttle_timer: ReturnType<typeof setTimeout> | null = null
    let throttle_pending: { wasm_op: WasmOp; payload: EvalPayload } | null = null
    let throttle_ready = true

    function _try_flush_throttle() {
        // console.log("try", throttle_pending, throttle_ready, status.value)
        if (throttle_pending === null) return
        if (status.value === "busy") return
        if (!throttle_ready) return

        const { wasm_op, payload } = throttle_pending
        throttle_pending = null
        throttle_ready = false

        throttle_timer = setTimeout(() => {
            throttle_ready = true
            _try_flush_throttle() // catch-up: new payload may have arrived during cooldown
        }, 1000)

        _launch(wasm_op, payload, false, () => _try_flush_throttle())
    }

    function throttled_start(wasm_op: WasmOp, payload: EvalPayload) {
        throttle_pending = { wasm_op, payload }
        _try_flush_throttle()
    }

    function _launch(wasm_op: WasmOp, payload: EvalPayload, cancel: boolean, callback?: (result) => void) {
        if (cancel) {
            cancel_worker()
        }

        status.value = "busy"
        error.value = null
        est_progress_percentage.value = 0
        if (!worker) {
            worker = createWorker()
        }

        worker.onmessage = (e) => {
            // console.log(e)

            if (e.data.type === "result") {
                result.value = e.data.result
                status.value = "success"
                est_progress_percentage.value = 100
                // console.log(mapToObject(toRaw(result.value)?.adv_cache) ?? null)
                if (cancel) {
                    worker.terminate()
                    worker = null
                }
            } else {
                if (performance.now() - last_intermediate_time.value > 1000) {
                    result.value = e.data.state_bundle
                    est_progress_percentage.value = e.data.est_progress_percentage
                    last_intermediate_time.value = performance.now()
                }
            }
            if (callback) {
                callback(result.value)
            }
        }

        worker.onerror = (e) => {
            error.value = e.message
            status.value = "error"
            worker = null
        }

        // console.log(WasmOp[wasm_op], payload)
        // console.log(JSON.parse(JSON.stringify(toRaw(buildPayload(wasm_op)))))
        worker.postMessage({ type: "message", wasm_op, payload })
    }

    function start(wasm_op: WasmOp, payload: EvalPayload, callback?: (result) => void) {
        // if (debounce > 0) {
        clearTimeout(debounceTimer)
        status.value = "busy"
        debounceTimer = setTimeout(() => _launch(wasm_op, payload, true, callback), 250)
        // } else {
        // _launch(wasm_op, payload, true, callback)
        // }
    }

    function cancel_worker() {
        if (worker) {
            worker.terminate()
            worker = null
            status.value = "idle"
        }
    }

    function cancel() {
        clearTimeout(debounceTimer)
        clearTimeout(throttle_timer)
        debounceTimer = null
        throttle_timer = null
        throttle_pending = null
        throttle_ready = true
        cancel_worker()
    }
    function cancel_and_clear_prev_result() {
        cancel()
        result.value = null
        est_progress_percentage.value = 0
    }

    onUnmounted(cancel)

    return { status, result, error, est_progress_percentage, start, throttled_start, cancel, cancel_and_clear_prev_result }
}
