import { ref } from "vue";
import type { CharProfile } from "@/Stores/CharacterProfile";
import { states_payload, type Payload } from "./PayloadBuilder";
import { create_worker_bundle } from "./WorkerBundle";
import { WasmOp } from "./WasmWorker";

// Optimizer runs off the calculator page, each followed by a histogram of what it chose, two at a
// time. One page queues at a time.

export interface Job {
  hash: string;
  payload: Payload;
  profile: CharProfile;
  // the histogram's output and the optimizer's state it was run with
  store: (out: any, state: any) => void;
}

// cyrb53
export function hash(text: string) {
  let h1 = 0xdeadbeef;
  let h2 = 0x41c6ce57;
  for (let i = 0; i < text.length; i++) {
    const ch = text.charCodeAt(i);
    h1 = Math.imul(h1 ^ ch, 2654435761);
    h2 = Math.imul(h2 ^ ch, 1597334677);
  }
  h1 = Math.imul(h1 ^ (h1 >>> 16), 2246822507);
  h1 ^= Math.imul(h2 ^ (h2 >>> 13), 3266489909);
  h2 = Math.imul(h2 ^ (h2 >>> 16), 2246822507);
  h2 ^= Math.imul(h1 ^ (h1 >>> 13), 3266489909);
  return (4294967296 * (2097151 & h2) + (h1 >>> 0)).toString(36);
}

export const jobs_left = ref(0);
let queue: Job[] = [];
const running = new Set<string>();
const workers = [create_worker_bundle(), create_worker_bundle()];
const idle = workers.map(() => true);

const count = () => (jobs_left.value = queue.length + running.size);

// the jobs wanted now, in order; those already running carry on
export function queue_jobs(jobs: Job[]) {
  queue = jobs.filter((job) => !running.has(job.hash));
  count();
  workers.forEach((_, at) => next(at));
}

export function stop_jobs() {
  queue = [];
  running.clear();
  workers.forEach((worker, at) => {
    worker.cancel();
    idle[at] = true;
  });
  count();
}

function next(at: number) {
  if (!idle[at]) return;
  const job = queue.shift();
  if (!job) return;
  const worker = workers[at];
  idle[at] = false;
  running.add(job.hash);
  count();
  worker.debounced_start(
    WasmOp.OptimizeAverage,
    job.payload,
    (state) => {
      // interim states come while it is still busy
      if (worker.status !== "idle") return;
      worker.debounced_start(
        WasmOp.Histogram,
        states_payload(job.payload, job.profile, state),
        (out) => {
          if (!running.delete(job.hash)) return;
          job.store(out, state);
          idle[at] = true;
          count();
          next(at);
        },
        0,
      );
    },
    0,
  );
}
