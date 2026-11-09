import fs from "node:fs";
import path from "node:path";
import { describe, it } from "vitest";
import { uniformInt } from "pure-rand/distribution/uniformInt";
import { xoroshiro128plus } from "pure-rand/generator/xoroshiro128plus";
import { createPinia, setActivePinia } from "pinia";
import { new_char_profile } from "@/Stores/CharacterProfile";
import { grids_to_keyed, UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { ilevel } from "@/Utils/HoningUtil";
import { parse_locale_float } from "@/Utils/InputColumn";
import { has_band, projected, WEEK_MS } from "@/Utils/Income";
import { auto_income, config_of, granted } from "@/Utils/IncomeSources";
import { SPECIAL_LEAP_LABELS } from "@/Utils/Constants";
import { job_for } from "@/Components/Character/Income/IncomeJobs";
import type { CharSnapshot, Snapshot } from "@/Utils/Scans";

// Iteration 3's real cases (arena_data/lab/iteration3): a character's snapshots in every pull grouped into
// families (same grids, stock within TOL of its gold value), each family's latest as a cold single, each
// roster's character wanting the most as an income chain (now and 12 weeks ahead), and each character with
// SEQ_MIN families or more as a chain of them in order; all at streak length 1 and 10, rosters split 50/50.
// Run: pnpm exec vitest run --config scripts/arena/vitest.export_real3.config.ts
const PULLS = "ignored/analytics";
const OUT = "arena_data/scenarios/real3";
const SETS = "arena_data/sets";
// the date of the last pull these were made from; pull again if it is more than a week old
const PULLED = "2026-11-09";
const WEEKS = 12;
const TOL = 0.1;
const SEQ_MIN = 3;
const SEED = 3;

const plain = (payload: any) =>
  JSON.parse(
    JSON.stringify({ ...payload, adv_cache: null }, (_, v) =>
      v instanceof Map ? Object.fromEntries(v) : v,
    ),
  );
const write = (name: string, body: object) =>
  fs.writeFileSync(path.join(OUT, `${name}.json`), JSON.stringify({ name, ...body }), {
    encoding: "utf8",
  });

const wants = (grid: number[][] = []) =>
  grid.flat().filter((x) => x === UpgradeStatus.Want).length;
const placeholder = (amounts: Record<string, number> = {}) =>
  Object.entries(amounts).some(
    ([label, amount]) =>
      label !== "Silver" && (/^9+$/.test(String(amount)) || amount >= 999_000_000),
  );
const cells = (grid: number[][] = [], keep: (v: number) => boolean) =>
  grid.flatMap((row, i) => row.flatMap((v, j) => (keep(v) ? [`${i},${j}`] : [])));
const done = (v: number) => v === UpgradeStatus.Done || v === UpgradeStatus.FetchedDone;
const want = (v: number) => v === UpgradeStatus.Want;
const grids_key = (char: CharSnapshot) =>
  JSON.stringify(
    [done, want].flatMap((keep) => [cells(char.normal_grid, keep), cells(char.adv_grid, keep)]),
  );
const price = (snapshot: Snapshot, label: string) => {
  const bundle = snapshot.bundles?.[label] || 1;
  return (snapshot.prices?.[`${label} x${bundle}`] || 0) / bundle;
};
function gold_of(snapshot: Snapshot, char: CharSnapshot) {
  const amounts: Record<string, number> = {};
  for (const band of [char.bound, snapshot.roster_owned, snapshot.tradable, char.special])
    for (const [label, amount] of Object.entries(band ?? {}))
      if (typeof amount === "number") amounts[label] = (amounts[label] ?? 0) + amount;
  return Object.fromEntries(
    Object.entries(amounts).map(([label, amount]) => [label, amount * price(snapshot, label)]),
  );
}
const moved = (a: Record<string, number>, b: Record<string, number>) => {
  const labels = new Set([...Object.keys(a), ...Object.keys(b)]);
  const total = Object.values(a).reduce((x, y) => x + y, 0) + 1e-9;
  let diff = 0;
  for (const label of labels) diff += Math.abs((a[label] ?? 0) - (b[label] ?? 0));
  return diff / total > TOL;
};

interface Case {
  snapshot: Snapshot;
  char_id: string;
  grids: string;
  gold: Record<string, number>;
}
interface Family {
  first: Case;
  latest: Case;
  members: number;
}

// per character, its families in order of their first snapshot
function families() {
  const rows = fs
    .readdirSync(PULLS)
    .filter((file) => file.startsWith("uploads-"))
    .sort()
    .flatMap((file) => JSON.parse(fs.readFileSync(path.join(PULLS, file), "utf8")))
    .filter((row) => row.version !== "dev" && !String(row.version).startsWith("followup"));
  const seen = new Set<string>();
  const by_char = new Map<string, Case[]>();
  for (const row of rows)
    for (const snapshot of row.snapshots as Snapshot[])
      for (const [char_id, char] of Object.entries(snapshot.chars)) {
        if (!char.normal_grid || !(wants(char.normal_grid) + wants(char.adv_grid))) continue;
        if (placeholder(char.bound) || placeholder(snapshot.roster_owned)) continue;
        const key = `${snapshot.roster_id} ${char_id}`;
        const content = JSON.stringify([
          char.bound,
          snapshot.roster_owned,
          snapshot.tradable,
          char.special,
          char.normal_grid,
          char.adv_grid,
          char.tier,
        ]);
        if (seen.has(`${key} ${content}`)) continue;
        seen.add(`${key} ${content}`);
        const one = { snapshot, char_id, grids: grids_key(char), gold: gold_of(snapshot, char) };
        by_char.set(key, [...(by_char.get(key) ?? []), one]);
      }
  const out = new Map<string, Family[]>();
  for (const [key, list] of by_char) {
    const fams: Family[] = [];
    for (const one of list.sort((a, b) => a.snapshot.at - b.snapshot.at)) {
      const fam = fams.find((f) => f.first.grids === one.grids && !moved(f.first.gold, one.gold));
      if (fam) {
        fam.latest = one;
        fam.members++;
      } else fams.push({ first: one, latest: one, members: 1 });
    }
    out.set(key, fams);
  }
  return out;
}

function profile_of(snapshot: Snapshot, char_id: string, streak: number) {
  const char = snapshot.chars[char_id];
  const profile = {
    ...new_char_profile(),
    id: char_id,
    roster_id: snapshot.roster_id,
    tier: char.tier,
    normal_grid: char.normal_grid,
    adv_grid: char.adv_grid,
    min_resolution: streak,
  };
  profile.keyed_upgrades = grids_to_keyed(char.normal_grid, char.adv_grid, {}, char.tier);
  for (const label of char.disabled ?? []) profile.bound_budgets.enabled[label] = false;
  return profile;
}

// the Income page's chain: now, then each week ahead at the Auto income of the item level
function income_steps(snapshot: Snapshot, char_id: string, streak: number) {
  const profile = profile_of(snapshot, char_id, streak);
  const item_level = parse_locale_float(ilevel(profile, "achieved")) || 0;
  const auto = auto_income(
    config_of(profile.income_sources, item_level),
    item_level,
    profile.tier,
    snapshot.at,
  );
  const rates = [0, 1, 2].map((band) =>
    Object.fromEntries(
      Object.entries(auto.rates[band]).filter(([label]) => has_band(label, band)),
    ),
  );
  const steps = [plain(job_for(profile, snapshot, true).payload)];
  for (let weeks = 1; weeks <= WEEKS; weeks++) {
    const ahead = projected(
      snapshot,
      char_id,
      SPECIAL_LEAP_LABELS[profile.tier],
      rates,
      weeks,
      granted(auto.grants, snapshot.at, snapshot.at + weeks * WEEK_MS),
    );
    steps.push(plain(job_for(profile, ahead, true).payload));
  }
  return { steps, item_level };
}

describe("export", () => {
  it("real3", () => {
    setActivePinia(createPinia());
    const fams = families();
    const rosters = [...new Set([...fams.keys()].map((key) => key.split(" ")[0]))].sort();
    const rng = xoroshiro128plus(SEED);
    for (let at = rosters.length - 1; at > 0; at--) {
      const other = uniformInt(rng, 0, at);
      [rosters[at], rosters[other]] = [rosters[other], rosters[at]];
    }
    const dev = new Set(rosters.slice(0, Math.ceil(rosters.length / 2)));
    const split = (roster: string) => (dev.has(roster) ? "dev" : "test");

    fs.rmSync(OUT, { recursive: true, force: true });
    fs.mkdirSync(OUT, { recursive: true });
    const sets: Record<string, string[]> = {};
    const add = (set: string, name: string) => (sets[set] = [...(sets[set] ?? []), name]);
    const count = (fam: Family) => {
      const char = fam.latest.snapshot.chars[fam.latest.char_id];
      return wants(char.normal_grid) + wants(char.adv_grid);
    };
    const meta_of = (fam: Family, roster: string, char_id: string, streak: number, payload: any) => ({
      source: "analytics",
      pulled: PULLED,
      roster: roster.slice(0, 8),
      char: char_id.slice(0, 6),
      split: split(roster),
      at: new Date(fam.latest.snapshot.at).toISOString().slice(0, 10),
      members: fam.members,
      tier: fam.latest.snapshot.chars[char_id].tier,
      upgrades: payload.upgrade_info.length,
      min_resolution: streak,
    });

    for (const [key, list] of [...fams.entries()].sort()) {
      const [roster, char_id] = key.split(" ");
      const stem = `${roster.slice(0, 8)}_${char_id.slice(0, 6)}`;
      for (const streak of [1, 10]) {
        // each family alone, cold
        const seq: any[] = [];
        list.forEach((fam, k) => {
          const payload = plain(
            job_for(profile_of(fam.latest.snapshot, char_id, streak), fam.latest.snapshot, true)
              .payload,
          );
          if (!payload.upgrade_info.length) return;
          const name = `r3_${stem}_f${k}_s${streak}`;
          write(name, { steps: [payload], meta: { ...meta_of(fam, roster, char_id, streak, payload), family: k } });
          add(`real3_${split(roster)}`, name);
          seq.push(payload);
        });
        // the character's families in order, chained
        if (seq.length >= SEQ_MIN) {
          const name = `r3seq_${stem}_s${streak}`;
          write(name, {
            steps: seq,
            meta: { ...meta_of(list[list.length - 1], roster, char_id, streak, seq[0]), families: seq.length },
          });
          add(`real3_seq_${split(roster)}`, name);
        }
      }
    }
    // per roster, the character whose latest family wants the most, as an income chain
    for (const roster of [...rosters].sort()) {
      const picks = [...fams.entries()]
        .filter(([key]) => key.startsWith(`${roster} `))
        .map(([key, list]) => ({ char_id: key.split(" ")[1], fam: list[list.length - 1] }))
        .sort((a, b) => count(b.fam) - count(a.fam) || a.char_id.localeCompare(b.char_id));
      const { char_id, fam } = picks[0];
      for (const streak of [1, 10]) {
        const { steps, item_level } = income_steps(fam.latest.snapshot, char_id, streak);
        if (!steps[0].upgrade_info.length) continue;
        const name = `r3inc_${roster.slice(0, 8)}_${char_id.slice(0, 6)}_s${streak}`;
        write(name, {
          steps,
          meta: {
            ...meta_of(fam, roster, char_id, streak, steps[0]),
            item_level,
            labels: ["now", ...steps.slice(1).map((_, w) => `+${w + 1}w`)],
          },
        });
        add(`real3_income_${split(roster)}`, name);
      }
    }

    const notes: Record<string, string> = {
      real3: "cold singles: each family's latest snapshot",
      real3_income: "income chains: each roster's character wanting the most, now and 12 weeks ahead at Auto income (chained)",
      real3_seq: "a character's families in order, 3 or more (chained)",
    };
    for (const [set, names] of Object.entries(sets)) {
      const kind = set.replace(/_(dev|test)$/, "");
      const body = [
        `# iteration 3 ${set.endsWith("dev") ? "dev" : "held out"}: ${notes[kind]}; real characters of the analytics pulls up to ${PULLED} (pull again if over a week old), families of a character's snapshots (same grids, stock within ${TOL * 100}% of its gold value), streak lengths 1 and 10 (_s1, _s10), rosters split 50/50 (scripts/arena/export_real3.test.ts)`,
        ...names.sort(),
      ];
      fs.writeFileSync(path.join(SETS, `${set}.txt`), body.join("\n") + "\n", { encoding: "utf8" });
      console.log(set, names.length);
    }
    console.log("rosters", rosters.length, "dev", dev.size, "families", [...fams.values()].reduce((n, l) => n + l.length, 0));
  });
});
