

/**
 * Flatten a 2D array (Array of Uint32Array rows) into one Uint32Array.
 * @param {Uint32Array[]} arr2d - Array of Uint32Array rows (all same length).
 * @returns {Uint32Array}
 */
export function flatten2DUint32(arr2d) {
  if (!arr2d || arr2d.length === 0) return new Uint32Array(0);

  const n = arr2d.length;
  const m = arr2d[0].length;

  // Validate shape quickly
  for (let i = 1; i < n; i++) {
    if (arr2d[i].length !== m) {
      throw new Error('All rows must have the same length');
    }
  }

  const out = new Uint32Array(n * m);
  let offset = 0;
  for (let i = 0; i < n; i++) {
    out.set(arr2d[i], offset); // fast bulk copy
    offset += m;
  }
  return out;
}
export async function clearObjectStore(dbName = 'my-cache-db', storeName = 'computed') {
  return new Promise((resolve, reject) => {
    const open = indexedDB.open('my-cache-db')
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {

        // const req = indexedDB.open('my-cache-db');
        // req.onerror = () => console.error('open error', req.error);
        // req.onblocked = () => console.warn('open blocked');
        // req.onupgradeneeded = (evt) => {
        // console.log('onupgradeneeded fired; oldVersion=', evt.oldVersion, 'newVersion=', evt.newVersion);
        // };
        // req.onsuccess = () => {
        // const db = req.result;
        // console.log('onsuccess: db.version=', db.version, 'objectStoreNames=', Array.from(db.objectStoreNames));
        // db.close();
        // };
      const db = open.result;
      console.log(db.objectStoreNames)
      if (!db.objectStoreNames.contains(storeName)) {
        db.close();
        console.warn(`object store "${storeName}" does not exist in DB "${dbName}"`);
        return resolve(false); // nothing to clear
      }
      const tx = db.transaction(storeName, 'readwrite');
      const store = tx.objectStore(storeName);
      const rreq = store.clear();
      rreq.onsuccess = () => { db.close(); resolve(true); };
      rreq.onerror = () => { db.close(); reject(rreq.error); };
    };
  });
}

/**
 * Reconstruct a 2D array from a flattened Uint32Array.
 * @param {Uint32Array} flat - Flat Uint32Array containing n*m elements.
 * @param {number} n - Number of rows.
 * @param {{zeroCopy?: boolean}} [opts] - If zeroCopy=true (default), returns subarray views into flat; otherwise returns fresh copies.
 * @returns {Uint32Array[]} - Array of Uint32Array rows (length n).
 */
export function reconstruct1DTo2D(flat, n, opts = { zeroCopy: true }) {
  const zeroCopy = opts.zeroCopy !== undefined ? opts.zeroCopy : true;

  if (!Number.isInteger(n) || n < 0) throw new Error('n must be a non-negative integer');
  if (n === 0) return [];

  if (!(flat instanceof Uint32Array)) {
    // tolerate Array or other typed arrays by converting
    flat = new Uint32Array(flat);
  }

  if (flat.length % n !== 0) {
    throw new Error('flat.length must be divisible by n');
  }

  const m = flat.length / n;
  const rows = new Array(n);

  if (zeroCopy) {
    // subarray: O(1) per row, no new element copies (they share the underlying buffer)
    for (let i = 0; i < n; i++) {
      rows[i] = flat.subarray(i * m, (i + 1) * m);
    }
  } else {
    // deep copy: independent rows
    for (let i = 0; i < n; i++) {
      const r = new Uint32Array(m);
      r.set(flat.subarray(i * m, (i + 1) * m));
      rows[i] = r;
    }
  }

  return rows;
}


// Open DB and create object store "computed" (one-time)
function openDB(dbName = 'my-cache-db', version = 1) {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(dbName, version);
    req.onupgradeneeded = (evt) => {
      const db = evt.target.result;
      if (!db.objectStoreNames.contains('computed')) {
        // key = string hash of inputs, value = { key, dtype, shape, blob, meta }
        db.createObjectStore('computed', { keyPath: 'key' });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

// Save a Float32Array (or any TypedArray) as a Blob with metadata
export async function saveTypedArray(key, typedArray, meta = {}, dbName = 'my-cache-db') {
  const db = await openDB(dbName);
  return new Promise((resolve, reject) => {
    const tx = db.transaction('computed', 'readwrite');
    const store = tx.objectStore('computed');

    // We store the ArrayBuffer as a Blob to be safe across browsers
    const blob = new Blob([typedArray.buffer], { type: 'application/octet-stream' });

    const value = {
      key,
      dtype: typedArray.constructor.name, // e.g., "Float32Array"
      length: typedArray.length,
      meta,
      blob,
    };

    const req = store.put(value);
    req.onsuccess = () => resolve(true);
    req.onerror = () => reject(req.error);
  });
}

// Load typed array by key (returns a typed array instance)
export async function loadTypedArray(key, dbName = 'my-cache-db') {
  const db = await openDB(dbName);
  return new Promise((resolve, reject) => {
    const tx = db.transaction('computed', 'readonly');
    const store = tx.objectStore('computed');
    const req = store.get(key);
    req.onsuccess = async () => {
      const rec = req.result;
      if (!rec) return resolve(null);
      const blob = rec.blob;
      try {
        const ab = await blob.arrayBuffer();
        let arr;
        switch (rec.dtype) {
          case 'Float32Array': arr = new Float32Array(ab); break;
          case 'Float64Array': arr = new Float64Array(ab); break;
          case 'Int16Array': arr = new Int16Array(ab); break;
          case 'UInt32Array': arr = new Uint32Array(ab); break;
          // add more if needed
          default: arr = new Uint32Array(ab); // fallback
        }
        resolve({ arr, meta: rec.meta });
      } catch (e) { reject(e); }
    };
    req.onerror = () => reject(req.error);
  });
}



