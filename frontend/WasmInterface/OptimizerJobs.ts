import { ref } from "vue";
import type { CharProfile } from "@/Stores/CharacterProfile";
import { states_payload, type Payload } from "./PayloadBuilder";
import { create_worker_bundle } from "./WorkerBundle";
import { WasmOp } from "./WasmTypes";

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
// every job of the last queue_jobs by hash: one run serves all jobs with its hash
let wanted = new Map<string, Job[]>();
const workers = [create_worker_bundle(), create_worker_bundle()];
const idle = workers.map(() => true);

const count = () => (jobs_left.value = queue.length + running.size);

// the jobs wanted now, in order, once per hash; those already running carry on
export function queue_jobs(jobs: Job[]) {
  wanted = Map.groupBy(jobs, (job) => job.hash);
  const seen = new Set(running);
  queue = jobs.filter((job) => !seen.has(job.hash) && seen.add(job.hash));
  count();
  workers.forEach((_, at) => next(at));
}

export function stop_jobs() {
  queue = [];
  wanted = new Map();
  running.clear();
  workers.forEach((worker, at) => {
    worker.cancel();
    idle[at] = true;
  });
  count();
}

function next(at: number) {
  if (!idle[at]) return;
  let job = queue.shift();
  while (job && running.has(job.hash)) job = queue.shift();
  if (!job) return;
  const worker = workers[at];
  idle[at] = false;
  running.add(job.hash);
  count();
  // a failed run is dropped, not retried
  const done = (out: any, state: any) => {
    if (!running.delete(job.hash)) return;
    if (out)
      for (const one of new Set([job, ...(wanted.get(job.hash) ?? [])]))
        one.store(out, state);
    idle[at] = true;
    count();
    next(at);
  };
  worker.debounced_start(
    WasmOp.OptimizeAverage,
    job.payload,
    (state) => {
      if (!state) return done(null, null);
      // interim states come while it is still busy
      if (worker.status !== "idle") return;
      worker.debounced_start(
        WasmOp.Histogram,
        states_payload(job.payload, job.profile, state),
        (out) => done(out, state),
        0,
      );
    },
    0,
  );
}
