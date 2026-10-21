import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { job_for } from "@/Components/Character/Income/IncomeJobs";
import {
  base_job,
  option_job,
} from "@/Components/Character/Choices/ChoiceJobs";
import { PRESET_CHOICES } from "@/Utils/Choices";
import { budgets_of } from "@/Utils/Scans";
import { MAX_WEEKS } from "@/Utils/Income";
import { fake_out, fake_state, snap, want_upgrade } from "./helpers";

// IncomeJobs and ChoiceJobs build the jobs; the queue itself is in optimizer-jobs.test.ts.

let profile: ReturnType<typeof useRosterStore>["active_profile"];
beforeEach(() => {
  setActivePinia(createPinia());
  profile = useRosterStore().active_profile;
  want_upgrade(profile);
});

describe("IncomeJobs", () => {
  it("one hash per budgets and upgrades, whatever the time or kind", () => {
    const a = snap(1, profile.id, { Red: 1 });
    const b = snap(2, profile.id, { Red: 1 }, { kind: "After export" });
    expect(job_for(profile, a, false).hash).toBe(
      job_for(profile, b, false).hash,
    );
    expect(
      job_for(profile, snap(1, profile.id, { Red: 2 }), false).hash,
    ).not.toBe(job_for(profile, a, false).hash);
    const before = job_for(profile, a, false).hash;
    want_upgrade(profile, 11);
    expect(job_for(profile, a, false).hash).not.toBe(before);
  });

  it("stores the result in the snapshot, the latest four kept", () => {
    const snapshot = snap(1, profile.id, { Red: 3 });
    for (let i = 0; i < 5; i++) {
      want_upgrade(profile, 10 + i);
      const job = job_for(profile, snapshot, false);
      expect(job.result).toBeUndefined();
      job.store(fake_out(100 + i, 40), fake_state());
      expect(job_for(profile, snapshot, false).result).toEqual({
        gains: {},
        gold: 100 + i,
        leftover: 60 + i,
      });
    }
    const results = Object.values(snapshot.chars[profile.id].results);
    expect(results.map((r) => r.gold)).toEqual([101, 102, 103, 104]);
  });

  it("a result stored twice is kept once, not pushing out an older one", () => {
    const snapshot = snap(1, profile.id, { Red: 3 });
    for (let i = 0; i < 3; i++) {
      want_upgrade(profile, 10 + i);
      job_for(profile, snapshot, false).store(
        fake_out(100 + i, 40),
        fake_state(),
      );
    }
    want_upgrade(profile, 13);
    const job = job_for(profile, snapshot, false);
    job.store(fake_out(200, 40), fake_state());
    job.store(fake_out(200, 40), fake_state());
    const results = Object.values(snapshot.chars[profile.id].results);
    expect(results.map((r) => r.gold)).toEqual([100, 101, 102, 200]);
  });

  it("gains follow the run's chest states", () => {
    const snapshot = snap(1, profile.id, {});
    snapshot.chars[profile.id].chests = [
      {
        key: "k",
        title: "C",
        band: 0,
        count: 3,
        options: [{ Red: 10 }, { Blue: 5 }],
        obtain_all: false,
      },
    ];
    const job = job_for(profile, snapshot, false);
    const key = Object.keys(job.payload.chests)[0];
    job.store(fake_out(1, 1), {
      ...fake_state(),
      chest_states: new Map([[key, [1, 2]]]),
    });
    expect(job_for(profile, snapshot, false).result.gains).toEqual({
      Blue: [5, 0, 0],
      Red: [20, 0, 0],
    });
  });

  it("now's results are kept in the profile, not in the snapshot", () => {
    const snapshot = snap(1, profile.id, { Red: 4 });
    const job = job_for(profile, snapshot, true);
    job.store(fake_out(9, 1), fake_state());
    expect(snapshot.chars[profile.id].results).toBeUndefined();
    expect(profile.income_results[job.hash].gold).toBe(9);
    expect(job_for(profile, snapshot, true).result.gold).toBe(9);
    expect(job_for(profile, snapshot, false).result).toBeUndefined();
  });

  it("the profile keeps a year and a week of them, newest last", () => {
    const hashes = Array.from({ length: MAX_WEEKS + 5 }, (_, at) => {
      const job = job_for(profile, snap(1, profile.id, { Red: at }), true);
      job.store(fake_out(at, 1), fake_state());
      return job.hash;
    });
    expect(Object.keys(profile.income_results)).toEqual(
      hashes.slice(-(MAX_WEEKS + 1)),
    );
  });

  // characterised: a run of a character the snapshot did not have writes it an empty entry
  it("a character missing from the snapshot gets an entry", () => {
    const snapshot = snap(1, "someone else", { Red: 1 });
    const before = budgets_of(snapshot);
    job_for(profile, snapshot, false).store(fake_out(1, 1), fake_state());
    expect(snapshot.chars[profile.id]).toMatchObject({
      bound: {},
      special: {},
      chests: [],
    });
    expect(budgets_of(snapshot)).not.toBe(before);
  });
});

describe("ChoiceJobs", () => {
  const choice = PRESET_CHOICES.find((x) => x.id === "bonus 202124 1");

  it("an option goes on from the run without once that is done", () => {
    const now = snap(1, profile.id, { Gold: 50000 });
    const base = base_job(profile, now);
    expect(base.metric).toBeUndefined();
    const early = option_job(profile, now, base, choice, 1);
    expect(early.payload.chest_states).toBeUndefined();
    expect(early.hash).not.toBe(base.hash);

    const state = fake_state(3);
    base.store(fake_out(0, 1234), state);
    expect(base_job(profile, now).metric).toBe(1234);
    const seeded = option_job(profile, now, base, choice, 1);
    expect(seeded.hash).toBe(early.hash);
    expect(seeded.payload.special_state).toEqual([3]);
    expect(Object.keys(seeded.payload.chests)).toHaveLength(1);
  });

  it("an option's run sets its metric, not a seed", () => {
    const now = snap(1, profile.id, { Gold: 60000 });
    const base = base_job(profile, now);
    base.store(fake_out(0, 10), fake_state(1));
    option_job(profile, now, base, choice, 1).store(
      fake_out(0, 25),
      fake_state(2),
    );
    expect(option_job(profile, now, base, choice, 1).metric).toBe(25);
    // the next option still goes on from the run without
    expect(
      option_job(profile, now, base, choice, 1).payload.special_state,
    ).toEqual([1]);
  });
});
