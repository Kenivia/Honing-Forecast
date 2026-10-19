// Enough of a browser for the store and persistence code to run under node.
const store = new Map();
globalThis.localStorage = {
  getItem: (k) => (store.has(k) ? store.get(k) : null),
  setItem: (k, v) => store.set(k, String(v)),
  removeItem: (k) => store.delete(k),
  clear: () => store.clear(),
};

// WasmWorker registers a message listener on `self` at module scope
globalThis.self = globalThis;
globalThis.addEventListener = () => {};

// Worker bundles are started by store actions we exercise; the runs themselves are not
// under test here, so a worker that never replies is enough.
globalThis.Worker = class {
  postMessage() {}
  terminate() {}
  addEventListener() {}
};
