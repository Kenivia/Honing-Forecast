import { SpawnWorker } from "@/WasmInterface/worker_setup.ts"

export type WasmOp = "EvaluateAverage" | "OptimizeAverage" | "Histogram"

export function runWasmOperation(payload: any, operation: WasmOp, onIntermediateMessage?: (_msg: any) => void) {
    return SpawnWorker(payload, operation, onIntermediateMessage)
}
