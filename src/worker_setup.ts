/* eslint-disable no-unused-vars */
const worker = new Worker(new URL("./js_to_wasm.js", import.meta.url), { type: "module" })

let nextId = 1
type Pending = {
    resolve: (...args: unknown[]) => void
    reject: (...args: unknown[]) => void
}
const pending = new Map<number, Pending>()

worker.addEventListener("message", (_ev: MessageEvent) => {
    const msg = _ev.data as { id?: number; type?: string; result?: unknown; error?: unknown }
    if (msg?.id && pending.has(msg.id)) {
        const { resolve: _resolve, reject: _reject } = pending.get(msg.id)!
        pending.delete(msg.id)
        if (msg.type === "result") _resolve(msg.result)
        else if (msg.type === "error") _reject(msg.error)
        else _resolve(msg)
    }
})

export function CallWorker(payload: unknown, which_one?: string) {
    return new Promise((resolve, reject) => {
        const id = nextId++
        pending.set(id, { resolve, reject })
        worker.postMessage({ id, payload, which_one })
    })
}
