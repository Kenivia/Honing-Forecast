import { chest_gains, process_chests } from "@/Utils/Chest";
import { char_chests, MAX_WEEKS } from "@/Utils/Income";
import type { Snapshot, SnapshotResult } from "@/Utils/Scans";
import type { CharProfile } from "@/Stores/CharacterProfile";
import {
  snapshot_payload,
  states_payload,
} from "@/WasmInterface/PayloadBuilder";
import { hash, type Job } from "@/WasmInterface/OptimizerJobs";

// Optimizer runs of snapshots under the current upgrades. A snapshot's results are kept in it by
// payload hash; those of now and the weeks ahead in the profile. The hash leaves out the state a
// run starts from, so a result holds whichever run came before it.

export interface SnapshotJob extends Job {
  result?: SnapshotResult;
}

// how many results a snapshot keeps per character, and a profile of now and the weeks ahead
const KEEP = 4;
const KEEP_AHEAD = MAX_WEEKS + 1;
// `results` with `key` last, the oldest dropped past `keep` (a run can be stored twice)
const kept = (
  results: Record<string, SnapshotResult> | undefined,
  key: string,
  result: SnapshotResult,
  keep: number,
) =>
  Object.fromEntries([
    ...Object.entries(results ?? {})
      .filter(([at]) => at !== key)
      .slice(1 - keep),
    [key, result],
  ]);
// each run's optimizer state, for the next in the chain to start from
const states = new Map<string, any>();
export const state_of = (hash: string) => states.get(hash);

export const warm = (job: SnapshotJob, from: any): SnapshotJob =>
  from
    ? { ...job, payload: states_payload(job.payload, job.profile, from) }
    : job;

export function job_for(
  profile: CharProfile,
  snapshot: Snapshot,
  // now or a week ahead, kept in the profile
  ahead: boolean,
): SnapshotJob {
  const char = snapshot.chars[profile.id];
  const chests = process_chests(
    char_chests(snapshot, profile.id),
    profile.tier,
  );
  const payload = snapshot_payload(profile, snapshot, chests);
  const key = hash(JSON.stringify(payload));
  return {
    hash: key,
    payload,
    profile,
    result: ahead ? profile.income_results[key] : char?.results?.[key],
    store: (out, state) => {
      const result: SnapshotResult = {
        gains: chest_gains(chests, state.chest_states),
        gold: out.metric_per_plan[0],
        leftover: out.metric_per_plan[0] - out.metric_per_plan[1],
      };
      states.set(key, state);
      if (ahead) {
        profile.income_results = kept(
          profile.income_results,
          key,
          { ...result, chance_within: out.chance_within },
          KEEP_AHEAD,
        );
        return;
      }
      const entry = (snapshot.chars[profile.id] ??= {
        name: profile.char_name,
        bound: {},
        special: {},
        chests: [],
      });
      entry.results = kept(entry.results, key, result, KEEP);
    },
  };
}
