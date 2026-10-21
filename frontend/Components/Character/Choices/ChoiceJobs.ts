import { shallowReactive } from "vue";
import { process_chests, type ManifestChest } from "@/Utils/Chest";
import { as_chest, type Choice } from "@/Utils/Choices";
import { char_chests } from "@/Utils/Income";
import type { Snapshot } from "@/Utils/Scans";
import type { CharProfile } from "@/Stores/CharacterProfile";
import {
  snapshot_payload,
  states_payload,
} from "@/WasmInterface/PayloadBuilder";
import { hash, type Job } from "@/WasmInterface/OptimizerJobs";

// The budgets as they are, and with each option of a choice taken, each optimized; an option's run
// goes on from the plan of the run without it. Kept for the session by payload hash.

export interface ChoiceJob extends Job {
  // the user's plan's metric, and the tradable gold used (plan 0, which credits no leftovers)
  metric?: number;
  gold?: number;
}

const metrics = shallowReactive(new Map<string, number>());
const golds = shallowReactive(new Map<string, number>());
// the optimizer's state of each run without a choice, by payload hash
const seeds = new Map<string, any>();

function job_with(
  profile: CharProfile,
  snapshot: Snapshot,
  extra: ManifestChest[],
  seed?: any,
): ChoiceJob {
  const payload = snapshot_payload(
    profile,
    snapshot,
    process_chests(
      [...char_chests(snapshot, profile.id), ...extra],
      profile.tier,
    ),
  );
  const key = hash(JSON.stringify(payload));
  return {
    hash: key,
    payload: seed ? states_payload(payload, profile, seed) : payload,
    profile,
    metric: metrics.get(key),
    gold: golds.get(key),
    store: (out, state) => {
      if (!extra.length) seeds.set(key, state);
      golds.set(key, out.metric_per_plan[0]);
      metrics.set(key, out.metric_per_plan[1]);
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
  job_with(profile, snapshot, [as_chest(choice, option)], seeds.get(base.hash));
