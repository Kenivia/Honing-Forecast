import { process_chests, type ManifestChest } from "@/Utils/Chest";
import { as_chest, type Choice } from "@/Utils/Choices";
import { char_chests } from "@/Utils/Income";
import type { Snapshot } from "@/Utils/Scans";
import type { CharProfile } from "@/Stores/CharacterProfile";
import {
  cached_state,
  fed_state,
  snapshot_payload,
  states_payload,
} from "@/WasmInterface/PayloadBuilder";
import { kept, payload_hash, type Job } from "@/WasmInterface/OptimizerJobs";

// The budgets as they are, and with each option of a choice taken, each optimized; an option's run
// goes on from the plan of the run without it. Kept in the profile by payload hash.

export interface ChoiceJob extends Job {
  // the user's plan's metric, and the tradable gold used (plan 0, which credits no leftovers)
  metric?: number;
  gold?: number;
}

// how many runs a profile keeps
const KEEP = 300;

function job_with(
  profile: CharProfile,
  snapshot: Snapshot,
  extra: ManifestChest[],
  seed?: CharProfile["choice_seed"],
): ChoiceJob {
  const payload = snapshot_payload(
    profile,
    snapshot,
    process_chests(
      [...char_chests(snapshot, profile.id), ...extra],
      profile.tier,
    ),
  );
  const key = payload_hash(payload);
  const result = profile.choice_results[key];
  return {
    hash: key,
    payload: seed
      ? states_payload(payload, profile, fed_state(seed.state))
      : payload,
    profile,
    gold: result?.[0],
    metric: result?.[1],
    store: (out, state) => {
      if (!extra.length) profile.choice_seed = cached_state(key, state);
      profile.choice_results = kept(
        profile.choice_results,
        key,
        [out.metric_per_plan[0], out.metric_per_plan[1]],
        KEEP,
      );
    },
  };
}

export const base_job = (profile: CharProfile, snapshot: Snapshot) =>
  job_with(profile, snapshot, []);

export const option_job = (
  profile: CharProfile,
  snapshot: Snapshot,
  base: ChoiceJob,
  choice: Choice,
  option: number,
) =>
  job_with(
    profile,
    snapshot,
    [as_chest(choice, option)],
    profile.choice_seed?.hash === base.hash ? profile.choice_seed : undefined,
  );
