import { shallowReactive } from "vue";
import { chest_gains, process_chests } from "@/Utils/Chest";
import { char_chests } from "@/Utils/Income";
import type { Snapshot, SnapshotResult } from "@/Utils/Scans";
import type { CharProfile } from "@/Stores/CharacterProfile";
import { payload_for } from "@/WasmInterface/PayloadBuilder";
import { hash, type Job } from "@/WasmInterface/OptimizerJobs";

// Optimizer runs of snapshots under the current upgrades. A snapshot's results are kept in it by
// payload hash; those of now only for the session.

export interface SnapshotJob extends Job {
  result?: SnapshotResult;
}

// how many results a snapshot keeps per character
const KEEP = 4;
const now_results = shallowReactive(new Map<string, SnapshotResult>());

export function job_for(
  profile: CharProfile,
  snapshot: Snapshot,
  is_now: boolean,
): SnapshotJob {
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
    profile,
    result: is_now ? now_results.get(key) : char?.results?.[key],
    store: (out, state) => {
      const result: SnapshotResult = {
        gains: chest_gains(chests, state.chest_states),
        gold: out.metric_per_plan[0],
        leftover: out.metric_per_plan[0] - out.metric_per_plan[1],
      };
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
