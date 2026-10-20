import { ref, shallowReactive } from "vue";
import { chest_gains, process_chests, type Chest } from "@/Utils/Chest";
import { char_chests } from "@/Utils/Income";
import type { Snapshot, SnapshotResult } from "@/Utils/Scans";
import type { CharProfile } from "@/Stores/CharacterProfile";
import {
  payload_for,
  states_payload,
  type Payload,
} from "@/WasmInterface/PayloadBuilder";
import { create_worker_bundle } from "@/WasmInterface/WorkerBundle";
import { WasmOp } from "@/WasmInterface/WasmWorker";

// Optimizer runs of snapshots under the current upgrades, two at a time. A snapshot's results are
// kept in it by payload hash; those of now only for the session.

export interface Job {
  hash: string;
  payload: Payload;
  chests: Chest[];
  profile: CharProfile;
  result?: SnapshotResult;
  store: (result: SnapshotResult) => void;
}

// how many results a snapshot keeps per character
const KEEP = 4;
const now_results = shallowReactive(new Map<string, SnapshotResult>());

// cyrb53
function hash(text: string) {
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

export function job_for(
  profile: CharProfile,
  snapshot: Snapshot,
  is_now: boolean,
): Job {
  const char = snapshot.chars[profile.id];
  const chests = process_chests(
    char_chests(snapshot, profile.id),
    profile.tier,
  );
  const payload = payload_for(
    profile,
    char?.bound ?? {},
    snapshot.roster_owned,
    snapshot.tradable,
    char?.special ?? {},
    chests,
    snapshot.prices,
    snapshot.bundles,
  );
  const key = hash(JSON.stringify(payload));
  return {
    hash: key,
    payload,
    chests,
    profile,
    result: is_now ? now_results.get(key) : char?.results?.[key],
    store: (result) => {
      if (is_now) {
        now_results.set(key, result);
        return;
      }
      const entry = (snapshot.chars[profile.id] ??= {
        name: profile.char_name,
        bound: {},
        special: {},
        chests: [],
      });
      const kept = Object.entries(entry.results ?? {}).slice(1 - KEEP);
      entry.results = Object.fromEntries([...kept, [key, result]]);
    },
  };
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
          job.store({
            gains: chest_gains(job.chests, state.chest_states),
            gold: out.metric_per_plan[0],
            leftover: out.metric_per_plan[0] - out.metric_per_plan[1],
          });
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
