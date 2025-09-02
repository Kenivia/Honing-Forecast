type TypedArray = Float32Array | Float64Array | Int16Array | Int32Array | Int8Array | Uint32Array | Uint16Array | Uint8Array | Uint8ClampedArray

export function flatten2DUint32(arr2d: Uint32Array[]): Uint32Array {
    if (!arr2d || arr2d.length === 0) return new Uint32Array(0)

    const n = arr2d.length
    const m = arr2d[0].length

    // Validate shape quickly
    for (let i = 1; i < n; i++) {
        if (arr2d[i].length !== m) {
            throw new Error("All rows must have the same length")
        }
    }

    const out = new Uint32Array(n * m)
    let offset = 0
    for (let i = 0; i < n; i++) {
        out.set(arr2d[i], offset) // fast bulk copy
        offset += m
    }
    return out
}

export async function clearObjectStore(dbName = "my-cache-db", storeName = "computed"): Promise<boolean> {
    return new Promise((resolve, reject) => {
        const openReq = indexedDB.open(dbName)
        openReq.onerror = () => reject(openReq.error)
        openReq.onsuccess = () => {
            const db = openReq.result as IDBDatabase
            if (!db.objectStoreNames.contains(storeName)) {
                db.close()
                console.warn(`object store "${storeName}" does not exist in DB "${dbName}"`)
                return resolve(false) // nothing to clear
            }
            const tx = db.transaction(storeName, "readwrite")
            const store = tx.objectStore(storeName)
            const rreq = store.clear()
            rreq.onsuccess = () => {
                db.close()
                resolve(true)
            }
            rreq.onerror = () => {
                db.close()
                reject(rreq.error)
            }
        }
    })
}

export function reconstruct1DTo2D(flat: Uint32Array | ArrayLike<number>, n: number, opts: { zeroCopy?: boolean } = { zeroCopy: true }): Uint32Array[] {
    const zeroCopy = opts.zeroCopy !== undefined ? opts.zeroCopy : true

    if (!Number.isInteger(n) || n < 0) throw new Error("n must be a non-negative integer")
    if (n === 0) return []

    // Ensure we have a concrete Uint32Array to call subarray on
    const typedFlat: Uint32Array = flat instanceof Uint32Array ? flat : new Uint32Array(flat as ArrayLike<number>)

    if (typedFlat.length % n !== 0) {
        throw new Error("flat.length must be divisible by n")
    }

    const m = typedFlat.length / n
    const rows: Uint32Array[] = new Array(n)

    if (zeroCopy) {
        // subarray: O(1) per row, no new element copies (they share the underlying buffer)
        for (let i = 0; i < n; i++) {
            rows[i] = typedFlat.subarray(i * m, (i + 1) * m)
        }
    } else {
        // deep copy: independent rows
        for (let i = 0; i < n; i++) {
            const r = new Uint32Array(m)
            r.set(typedFlat.subarray(i * m, (i + 1) * m))
            rows[i] = r
        }
    }

    return rows
}

// Open DB and create object store "computed" (one-time)
function openDB(dbName = "my-cache-db", version = 1): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
        const req = indexedDB.open(dbName, version)
        req.onupgradeneeded = (evt) => {
            const db = (evt.target as IDBOpenDBRequest).result
            if (!db.objectStoreNames.contains("computed")) {
                // key = string hash of inputs, value = { key, dtype, shape, blob, meta }
                db.createObjectStore("computed", { keyPath: "key" })
            }
        }
        req.onsuccess = () => resolve(req.result)
        req.onerror = () => reject(req.error)
    })
}

// Save a Float32Array (or any TypedArray) as a Blob with metadata
export async function saveTypedArray(key: string, typedArray: TypedArray, meta: Record<string, any> = {}, dbName = "my-cache-db"): Promise<boolean> {
    const db = await openDB(dbName)
    return new Promise((resolve, reject) => {
        const tx = db.transaction("computed", "readwrite")
        const store = tx.objectStore("computed")

        // We store the ArrayBuffer as a Blob to be safe across browsers.
        // Cast the buffer to ArrayBuffer so it satisfies the BlobPart types in lib.dom
        const blob = new Blob([typedArray.buffer as unknown as ArrayBuffer], { type: "application/octet-stream" })

        const value = {
            key,
            dtype: typedArray.constructor.name, // e.g., "Float32Array"
            length: typedArray.length,
            meta,
            blob,
        }

        const req = store.put(value)
        req.onsuccess = () => resolve(true)
        req.onerror = () => reject(req.error)
    })
}

// Load typed array by key (returns a typed array instance)
export async function loadTypedArray(key: string, dbName = "my-cache-db"): Promise<{ arr: TypedArray; meta: any } | null> {
    const db = await openDB(dbName)
    return new Promise((resolve, reject) => {
        const tx = db.transaction("computed", "readonly")
        const store = tx.objectStore("computed")
        const req = store.get(key)
        req.onsuccess = async () => {
            const rec = req.result
            if (!rec) return resolve(null)
            const blob: Blob = rec.blob
            try {
                const ab = await blob.arrayBuffer()
                let arr: TypedArray
                switch (rec.dtype) {
                    case "Float32Array":
                        arr = new Float32Array(ab)
                        break
                    case "Float64Array":
                        arr = new Float64Array(ab)
                        break
                    case "Int16Array":
                        arr = new Int16Array(ab)
                        break
                    case "Int32Array":
                        arr = new Int32Array(ab)
                        break
                    case "Int8Array":
                        arr = new Int8Array(ab)
                        break
                    case "Uint8Array":
                        arr = new Uint8Array(ab)
                        break
                    case "Uint16Array":
                        arr = new Uint16Array(ab)
                        break
                    case "Uint32Array":
                        arr = new Uint32Array(ab)
                        break
                    case "Uint8ClampedArray":
                        arr = new Uint8ClampedArray(ab)
                        break
                    default:
                        arr = new Uint32Array(ab) // fallback
                }
                resolve({ arr, meta: rec.meta })
            } catch (e) {
                reject(e)
            }
        }
        req.onerror = () => reject(req.error)
    })
}
