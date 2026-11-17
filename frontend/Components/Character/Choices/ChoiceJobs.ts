import { as_chest, type Choice } from "@/Utils/Choices";
import type { ManifestChest } from "@/Utils/Chest";
import type { CharProfile } from "@/Stores/CharacterProfile";
import {
  cached_state,
  fed_state,
  states_payload,
  store_payload,
} from "@/WasmInterface/PayloadBuilder";
import { kept, payload_hash, type Job } from "@/WasmInterface/OptimizerJobs";

// The calculator's payload as it is, and with each option of a choice taken, each optimized; an
// option's run goes on from the plan of the run without it. The run without is the calculator's
// run: both share `optimizer_cache`. Results kept in the profile by payload hash.

export interface ChoiceJob extends Job {
  // the user's plan's metric, and the tradable gold used (plan 0, which credits no leftovers)
  metric?: number;
  gold?: number;
}

// how many runs a profile keeps
const KEEP = 300;

// the saved run of `hash`, if it is the last
const saved = (profile: CharProfile, hash: string) =>
  profile.optimizer_cache?.hash === hash
    ? fed_state(profile.optimizer_cache.state)
    : undefined;

function job_with(
  profile: CharProfile,
  extra: ManifestChest[],
  seed_hash?: string,
): ChoiceJob {
  const payload = store_payload(extra);
  const key = payload_hash(payload);
  const result = profile.choice_results[key];
  const seed = seed_hash && saved(profile, seed_hash);
  return {
    hash: key,
    // an option is the budgets with one chest more: a short search from the run without
    payload: seed
      ? { ...states_payload(payload, profile, seed), quick: true }
      : payload,
    profile,
    state: extra.length ? undefined : saved(profile, key),
    gold: result?.[0],
    metric: result?.[1],
    store: (out, state) => {
      if (!extra.length) profile.optimizer_cache = cached_state(key, state);
      profile.choice_results = kept(
        profile.choice_results,
        key,
        [out.metric_per_plan[0], out.metric_per_plan[1]],
        KEEP,
      );
    },
  };
}

export const base_job = (profile: CharProfile) => job_with(profile, []);

export const option_job = (
  profile: CharProfile,
  base: ChoiceJob,
  choice: Choice,
  option: number,
) => job_with(profile, [as_chest(choice, option)], base.hash);
