import { SpawnWorker } from "../../worker_setup.ts"
import { INPUT_LABELS } from "./constants.ts"

export function buildPayload({
    topGrid,
    bottomGrid,
    desired_chance,
    budget_inputs,
    adv_hone_strategy,
    express_event,
    bucketCount,
    autoOptimization,
    userMatsValue,
    dataSize,
}: {
    topGrid: boolean[][]
    bottomGrid: boolean[][]
    desired_chance: string
    budget_inputs: any
    adv_hone_strategy: string
    express_event: boolean
    bucketCount: string
    autoOptimization: boolean
    userMatsValue: any
    dataSize: string
}) {
    return {
        normal_hone_ticks: topGrid,
        adv_hone_ticks: bottomGrid,
        desired_chance: parseFloat(desired_chance || "0"),
        budget: ((input) => Object.entries(input).map(([, v]) => Math.round(Number(v))))(budget_inputs),
        adv_hone_strategy: adv_hone_strategy,
        express_event: express_event,
        bucket_count: Math.max(2, Math.min(1000, Math.floor(Number(bucketCount) || 2))),
        user_mats_value: autoOptimization
            ? INPUT_LABELS.slice(0, 7).map((_) => 0.0)
            : INPUT_LABELS.slice(0, 7).map((label) => parseFloat(userMatsValue[label] || "0")),
        data_size: Math.max(1000, Math.floor(Number(dataSize) || 0)),
    }
}

export function createStartCancelableWorker({
    costWorkerRef,
    chanceWorkerRef,
    setCostToChanceBusy,
    setChanceToCostBusy,
    set_chance_result,
    set_cost_result,
    setCachedChanceGraphData,
    setCachedCostGraphData,
}: {
    costWorkerRef: React.MutableRefObject<Worker | null>
    chanceWorkerRef: React.MutableRefObject<Worker | null>
    setCostToChanceBusy: React.Dispatch<React.SetStateAction<boolean>>
    setChanceToCostBusy: React.Dispatch<React.SetStateAction<boolean>>
    set_chance_result: React.Dispatch<React.SetStateAction<any>>
    set_cost_result: React.Dispatch<React.SetStateAction<any>>
    setCachedChanceGraphData: React.Dispatch<React.SetStateAction<{ hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null>>
    setCachedCostGraphData: React.Dispatch<React.SetStateAction<{ hist_counts?: any; hist_mins?: any; hist_maxs?: any } | null>>
}) {
    return (which_one: "CostToChance" | "ChanceToCost", payload: any) => {
        if (which_one === "CostToChance") {
            // terminate previous
            if (costWorkerRef.current) {
                try {
                    costWorkerRef.current.terminate()
                } catch (e) {
                    /* ignore */
                }
                costWorkerRef.current = null
            }
            setCostToChanceBusy(true)
            // Clear text results but preserve graph data
            set_chance_result(null)

            const { worker, promise } = SpawnWorker(payload, which_one)
            costWorkerRef.current = worker

            promise
                .then((res) => {
                    // only set if this worker is still the current one
                    if (costWorkerRef.current === worker) {
                        set_chance_result(res)
                        // Cache graph data for future use
                        if (res && typeof res === "object" && res !== null && "hist_counts" in res) {
                            const typedRes = res as { hist_counts: any; hist_mins: any; hist_maxs: any }
                            setCachedChanceGraphData({
                                hist_counts: typedRes.hist_counts,
                                hist_mins: typedRes.hist_mins,
                                hist_maxs: typedRes.hist_maxs,
                            })
                        }
                    }
                })
                .catch((err) => {
                    console.error("Worker error", err)
                    if (costWorkerRef.current === worker) {
                        set_chance_result({ error: String(err) })
                    }
                })
                .finally(() => {
                    // cleanup if this worker is the current one
                    if (costWorkerRef.current === worker) {
                        try {
                            worker.terminate()
                        } catch (e) {
                            /* ignore */
                        }
                        costWorkerRef.current = null
                        setCostToChanceBusy(false)
                    }
                })
        } else {
            // ChanceToCost
            if (chanceWorkerRef.current) {
                try {
                    chanceWorkerRef.current.terminate()
                } catch (e) {
                    /* ignore */
                }
                chanceWorkerRef.current = null
            }
            setChanceToCostBusy(true)
            // Clear text results but preserve graph data
            set_cost_result(null)

            const { worker, promise } = SpawnWorker(payload, which_one)
            chanceWorkerRef.current = worker

            promise
                .then((res) => {
                    if (chanceWorkerRef.current === worker) {
                        set_cost_result(res)
                        // Cache graph data for future use
                        if (res && typeof res === "object" && res !== null && "hist_counts" in res) {
                            const typedRes = res as { hist_counts: any; hist_mins: any; hist_maxs: any }
                            setCachedCostGraphData({
                                hist_counts: typedRes.hist_counts,
                                hist_mins: typedRes.hist_mins,
                                hist_maxs: typedRes.hist_maxs,
                            })
                        }
                    }
                })
                .catch((err) => {
                    console.error("Worker error", err)
                    if (chanceWorkerRef.current === worker) {
                        set_cost_result({ error: String(err) })
                    }
                })
                .finally(() => {
                    if (chanceWorkerRef.current === worker) {
                        try {
                            worker.terminate()
                        } catch (e) {
                            /* ignore */
                        }
                        chanceWorkerRef.current = null
                        setChanceToCostBusy(false)
                    }
                })
        }
    }
}

export function createHandleCallWorker({
    startCancelableWorker,
    buildPayload,
}: {
    startCancelableWorker: (_which_one: "CostToChance" | "ChanceToCost", _payload: any) => void
    buildPayload: () => any
}) {
    return async (which_one: string) => {
        // keep the old behavior for manual button calls but make it cancel previous worker using startCancelableWorker
        const payload = buildPayload()
        if (which_one === "CostToChance") startCancelableWorker("CostToChance", payload)
        else startCancelableWorker("ChanceToCost", payload)
    }
}

export function createDebounceEffects({
    debounceTimerRef1,
    debounceTimerRef2,
    startCancelableWorker,
    buildPayload,
}: {
    debounceTimerRef1: React.MutableRefObject<number | null>
    debounceTimerRef2: React.MutableRefObject<number | null>
    startCancelableWorker: (_which_one: "CostToChance" | "ChanceToCost", _payload: any) => void
    buildPayload: () => any
}) {
    return {
        // When budget or grids or strategy change -> run CostToChance (budget -> cost->chance)
        createCostToChanceEffect: (_deps: any[]) => {
            return () => {
                // clear existing timer
                if (debounceTimerRef1.current) {
                    window.clearTimeout(debounceTimerRef1.current)
                    debounceTimerRef1.current = null
                }
                // start new delayed work
                debounceTimerRef1.current = window.setTimeout(() => {
                    const payload = buildPayload()
                    startCancelableWorker("CostToChance", payload)
                    debounceTimerRef1.current = null
                }, 100) // 100ms debounce
            }
        },

        // When desired chance or grids or strategy change -> run ChanceToCost (chance -> cost)
        createChanceToCostEffect: (_deps: any[]) => {
            return () => {
                if (debounceTimerRef2.current) {
                    window.clearTimeout(debounceTimerRef2.current)
                    debounceTimerRef2.current = null
                }
                debounceTimerRef2.current = window.setTimeout(() => {
                    const payload = buildPayload()
                    startCancelableWorker("ChanceToCost", payload)
                    debounceTimerRef2.current = null
                }, 100)
            }
        },
    }
}
