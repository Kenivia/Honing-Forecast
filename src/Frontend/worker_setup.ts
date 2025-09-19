// worker_setup.js  (replace existing SpawnWorker / CallWorker)
function makeId() {
    // simple unique id
    return Math.random().toString(36).slice(2) + Date.now().toString(36)
}

export function SpawnWorker(payload, which_one) {
    // adjust path if needed
    const worker = new Worker(new URL("./js_to_wasm.ts", import.meta.url), { type: "module" })
    const id = makeId()

    let settled = false
    const promise = new Promise((resolve, reject) => {
        const onMessage = (ev) => {
            // only accept messages that are result messages with the same id
            const msg = ev.data
            if (msg && msg.type === "result" && msg.id === id) {
                settled = true
                worker.removeEventListener("message", onMessage)
                worker.removeEventListener("error", onError)
                resolve(msg.result)
            } else {
                // ignore unrelated messages (or you could expose them as events)
            }
        }
        const onError = (err) => {
            if (!settled) {
                settled = true
                worker.removeEventListener("message", onMessage)
                worker.removeEventListener("error", onError)
                reject(err)
            }
        }

        worker.addEventListener("message", onMessage)
        worker.addEventListener("error", onError)

        // send the message including the id so worker can attach it to the response
        try {
            worker.postMessage({ id, payload, which_one })
        } catch (e) {
            // posting failed — clean up
            worker.removeEventListener("message", onMessage)
            worker.removeEventListener("error", onError)
            reject(e)
        }
    })

    // When the promise settles we'll terminate the worker for safety
    const wrapped = promise.finally(() => {
        try {
            worker.terminate()
        } catch (e) {
            //
        }
    })

    return { worker, promise: wrapped }
}

export async function CallWorker(payload, which_one) {
    const { worker, promise } = SpawnWorker(payload, which_one)
    try {
        const res = await promise
        try {
            worker.terminate()
        } catch (e) {
            //
        }
        return res
    } catch (err) {
        try {
            worker.terminate()
        } catch (e) {
            //
        }
        throw err
    }
}
