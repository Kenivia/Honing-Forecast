import { StatePair } from "@/Sections/Optimize/StateGrid.tsx"
import { SpawnWorker } from "./worker_setup.ts"
import { JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants.ts"
import type { InputsValues } from "@/Utils/InputBundles.ts"
// import { ticksToCounts } from "@/Utils/Helpers.ts"

export function buildPayload({
    topGrid,
    bottomGrid,
    adv_hone_strategy,
    express_event,
    bucketCount,
    // autoGoldValues,
    dataSize,
    inputs,
    progressGrid,
    unlockGrid,
    succeededGrid,
    stateBundleGrid,
    specialState,
    minResolution,
}: {
    topGrid: boolean[][]
    bottomGrid: boolean[][]
    adv_hone_strategy: string
    express_event: boolean
    bucketCount: string

    dataSize: string
    inputs: InputsValues
    progressGrid: number[][]
    unlockGrid: boolean[][]
    succeededGrid: boolean[][]
    stateBundleGrid: StatePair[][][]
    specialState: number[]
    minResolution: number
}) {
    const { mats, juice } = inputs
    // console.log(mats)
    const payload: any = {
        mats_budget: MATS_LABELS.slice(0, 8).map((label) => parseFloat(mats.owned[label] || "0")),
        adv_hone_strategy: adv_hone_strategy,
        express_event: express_event,
        bucket_count: Math.max(2, Math.min(1000, Math.floor(Number(bucketCount) || 2))),
        user_price_arr: MATS_LABELS.slice(0, 7).map((label) => parseFloat(mats.prices[label] || "0")),
        data_size: Math.max(1000, Math.floor(Number(dataSize) || 0)),
        inp_leftover_values: MATS_LABELS.slice(0, 7).map((label) => parseFloat(mats.leftover[label] || "0")),
        juice_books_budget: JUICE_LABELS.map((label_row) => [parseFloat(juice.weapon.owned[label_row[0]]), parseFloat(juice.armor.owned[label_row[1]])]),

        juice_prices: JUICE_LABELS.map((label_row) => [parseFloat(juice.weapon.prices[label_row[0]]), parseFloat(juice.armor.prices[label_row[1]])]),
        inp_leftover_juice_values: JUICE_LABELS.map((label_row) => [
            parseFloat(juice.weapon.leftover[label_row[0]]),
            parseFloat(juice.armor.leftover[label_row[1]]),
        ]),

        progress_grid: progressGrid,
        unlocked_grid: unlockGrid,
        succeeded_grid: succeededGrid,
        state_grid: stateBundleGrid,
        special_state: specialState,
        min_resolution: Math.max(1, Math.min(219, Math.floor(minResolution || 1))),
        num_threads: navigator.hardwareConcurrency,
    }

    // Always use the traditional tick-based approach
    payload.normal_hone_ticks = topGrid
    payload.adv_hone_ticks = bottomGrid
    console.log("payload:", payload)
    return payload
}

/** Optional shape for cached graph setters */
type CachedGraphSetter = React.Dispatch<React.SetStateAction<{ hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null>>

type BusySetter = React.Dispatch<React.SetStateAction<boolean>>
type ResultSetter = React.Dispatch<React.SetStateAction<any>>

/**
 * Options for a single start call.
 * All except `workerRef` and `setBusy`/`setResult` are optional.
 */
export type StartOptions = {
    which_one: string
    payloadBuilder: () => any
    workerRef: React.MutableRefObject<Worker | null>
    setBusy: BusySetter
    setResult: ResultSetter
    setCachedGraphData?: CachedGraphSetter
    // optional: extra success handler
    onSuccess?: (_) => void
    // optional: extra error handler

    onIntermediateMessage?: (_) => void
    onError?: (_) => void
    // optional: extra finally handler
    onFinally?: () => void
    // debounce key (defaults to which_one); different keys share different debounce timers
    debounceKey?: string
    // optional per-call debounce delay ms (if 0 or undefined, no debounce)
    debounceMs?: number
    dependency?: boolean
}

/**
 * Factory that returns `start` and `cancel` functions.
 * - `start(opts)` starts a worker (debounced if requested).
 * - `cancel(key)` cancels any pending debounce + terminates the live worker for that key.
 *
 * Uses an internal map keyed by debounceKey (or which_one) so you can independently debounce
 * different kinds of workers.
 */
export function createCancelableWorkerRunner() {
    // map of debounce timers keyed by string
    const timers = new Map<string, number>()
    // map of running workers keyed by string (so cancel(key) terminates only that worker)
    const runningWorkers = new Map<string, Worker>()
    function terminateWorkerSafe(worker: Worker | null) {
        if (!worker) return
        try {
            worker.terminate()
        } catch (e) {
            // ignore termination errors
        }
    }
    /**
     * Start a worker with the provided options.
     */
    function start(opts: StartOptions) {
        const {
            which_one,
            payloadBuilder,
            workerRef,
            setBusy,
            setResult,
            setCachedGraphData,
            onSuccess,
            onIntermediateMessage,
            onError,
            onFinally,
            debounceKey = which_one,
            debounceMs = 150,
            dependency = true,
        } = opts

        const runNow = () => {
            // Cancel/terminate any previous worker referenced by workerRef
            if (workerRef.current) {
                try {
                    workerRef.current.terminate()
                } catch (e) {
                    /* ignore */
                }
                workerRef.current = null
            }

            // mark as busy, clear previous result (but let caller decide cached graph preservation)
            // setResult(null)
            if (!dependency) {
                // console.log("tried", which_one, "but monte carlo wasn't ready")
                return
            }
            // console.log("actually started", which_one)
            setBusy(true)

            // spawn the worker (uses your existing SpawnWorker)
            const { worker, promise } = SpawnWorker(payloadBuilder(), which_one, onIntermediateMessage)

            // store refs so cancel(key) can terminate this worker specifically
            workerRef.current = worker
            runningWorkers.set(debounceKey, worker)

            promise
                .then((res) => {
                    // only act if this worker is still the current one
                    if (workerRef.current === worker) {
                        setResult(res)
                        // auto-cache graph-like data if present and setter provided
                        if (setCachedGraphData && res && typeof res === "object" && "hist_counts" in res) {
                            const typedRes = res as { hist_counts?: any; hist_mins?: any; hist_maxs?: any }
                            setCachedGraphData({
                                hist_counts: typedRes.hist_counts,
                                hist_mins: typedRes.hist_mins,
                                hist_maxs: typedRes.hist_maxs,
                            })
                        }
                        onSuccess?.(res)
                    }
                })
                .catch((err) => {
                    console.error("Worker error", err)
                    if (workerRef.current === worker) {
                        setResult({ error: String(err) })
                    }
                    onError?.(err)
                })
                .finally(() => {
                    // cleanup only if this worker is still current
                    if (workerRef.current === worker) {
                        try {
                            worker.terminate()
                        } catch (e) {
                            /* ignore */
                        }
                        workerRef.current = null
                        runningWorkers.delete(debounceKey)
                        setBusy(false)
                        onFinally?.()
                    }
                })
        }

        // If debounceMs provided and > 0, schedule with debounce
        if (debounceMs && debounceMs > 0) {
            // clear any existing timer for this key
            // console.log(which_one, "considered")
            const existingTimer = timers.get(debounceKey)
            if (existingTimer) {
                clearTimeout(existingTimer)
            }
            // setResult(null)
            // schedule new timer (window.setTimeout returns number)
            const t = window.setTimeout(() => {
                timers.delete(debounceKey)
                runNow()
            }, debounceMs)
            timers.set(debounceKey, t as unknown as number)
        } else {
            // no debounce requested — run immediately

            runNow()
        }
    }

    /**
     * Cancel pending debounce or running worker for a given key.
     * If no key provided, cancels everything.
     */
    function cancel(key?: string) {
        if (key) {
            const t = timers.get(key)
            if (t) {
                clearTimeout(t)
                timers.delete(key)
            }
            const w = runningWorkers.get(key)
            if (w) {
                terminateWorkerSafe(w)
                runningWorkers.delete(key)
            }
        } else {
            // clear all timers and terminate all running workers
            for (const t of timers.values()) clearTimeout(t)
            timers.clear()
            for (const w of runningWorkers.values()) terminateWorkerSafe(w)
            runningWorkers.clear()
        }
    }

    return { start, cancel }
}
