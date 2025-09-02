const worker = new Worker(new URL('./workers/MainWorker.js', import.meta.url), { type: 'module' });

let nextId = 1;
const pending = new Map();

worker.addEventListener('message', (ev) => {
    const msg = ev.data;
    if (msg?.id && pending.has(msg.id)) {
        const { resolve, reject } = pending.get(msg.id);
        pending.delete(msg.id);
        if (msg.type === 'result') resolve(msg.result);
        else if (msg.type === 'error') reject(msg.error);
        else resolve(msg);
    }
});

export function CallWorker(payload, which_one) {
    return new Promise((resolve, reject) => {
        const id = nextId++;
        pending.set(id, { resolve, reject });
        worker.postMessage({ id, payload, which_one });
    });
}

