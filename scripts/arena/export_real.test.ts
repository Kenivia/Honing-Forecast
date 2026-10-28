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
import type { Snapshot } from "@/Utils/Scans";

// One-off: held-out arena scenarios from real characters in the analytics pulls
// (ignored/analytics): each picked character's latest budgets as a single, and as an income chain
// of now and 12 weeks ahead at the Income page's Auto income; half of them at streak length 1. Run:
// pnpm exec vitest run --config scripts/arena/vitest.export_real.config.ts
const PULLS = "ignored/analytics";
const OUT = "arena_data/scenarios";
const WEEKS = 12;
// a roster gives at most this many singles; its first is also an income chain
const PER_ROSTER = 2;
// a seeded random half of the characters gets streak length 1, the rest the default 10
const SEED = 1;

const plain = (payload: any) =>
  JSON.parse(
    JSON.stringify({ ...payload, adv_cache: null }, (_, v) =>
      v instanceof Map ? Object.fromEntries(v) : v,
    ),
  );
const write = (dir: string, name: string, body: object) => {
  fs.mkdirSync(path.join(OUT, dir), { recursive: true });
  fs.writeFileSync(path.join(OUT, dir, `${name}.json`), JSON.stringify({ name, ...body }), {
    encoding: "utf8",
  });
};

const wants = (grid: number[][] = []) =>
  grid.flat().filter((x) => x === UpgradeStatus.Want).length;
// typed placeholders (a roster of 99,999 cells, a cell cap) say nothing of a real budget
const placeholder = (amounts: Record<string, number>) =>
  Object.entries(amounts).some(
    ([label, amount]) =>
      label !== "Silver" && (/^9+$/.test(String(amount)) || amount >= 999_000_000),
  );

function picks() {
  const rows = fs
    .readdirSync(PULLS)
    .filter((file) => file.startsWith("uploads-"))
    .flatMap((file) => JSON.parse(fs.readFileSync(path.join(PULLS, file), "utf8")))
    .filter((row) => row.version !== "dev" && !String(row.version).startsWith("followup"));
  // each character's latest snapshot with something wanted
  const latest = new Map<string, { snapshot: Snapshot; char_id: string }>();
  for (const row of rows)
    for (const snapshot of row.snapshots as Snapshot[])
      for (const [char_id, char] of Object.entries(snapshot.chars)) {
        if (!char.normal_grid || !(wants(char.normal_grid) + wants(char.adv_grid))) continue;
        if (placeholder(char.bound) || placeholder(snapshot.roster_owned)) continue;
        const key = `${snapshot.roster_id} ${char_id}`;
        if (!latest.has(key) || latest.get(key).snapshot.at < snapshot.at)
          latest.set(key, { snapshot, char_id });
      }
  // per roster, the character wanting the most and one wanting another amount, distinct budgets
  const by_roster = new Map<string, { snapshot: Snapshot; char_id: string }[]>();
  for (const pick of latest.values())
    by_roster.set(pick.snapshot.roster_id, [
      ...(by_roster.get(pick.snapshot.roster_id) ?? []),
      pick,
    ]);
  const seen = new Set<string>();
  const out: { snapshot: Snapshot; char_id: string; first: boolean }[] = [];
  for (const list of [...by_roster.values()].sort((a, b) =>
    a[0].snapshot.roster_id.localeCompare(b[0].snapshot.roster_id),
  )) {
    const count = (pick: (typeof list)[0]) => {
      const char = pick.snapshot.chars[pick.char_id];
      return wants(char.normal_grid) + wants(char.adv_grid);
    };
    list.sort((a, b) => count(b) - count(a) || a.char_id.localeCompare(b.char_id));
    const chosen = [list[0]];
    const other = list[Math.floor(list.length / 2)];
    if (PER_ROSTER > 1 && count(other) !== count(list[0])) chosen.push(other);
    chosen.forEach((pick, at) => {
      const char = pick.snapshot.chars[pick.char_id];
      const text = JSON.stringify([char.bound, char.normal_grid, char.adv_grid, char.tier]);
      if (seen.has(text)) return;
      seen.add(text);
      out.push({ ...pick, first: at === 0 });
    });
  }
  return out;
}

function profile_of(snapshot: Snapshot, char_id: string) {
  const char = snapshot.chars[char_id];
  const profile = {
    ...new_char_profile(),
    id: char_id,
    roster_id: snapshot.roster_id,
    tier: char.tier,
    normal_grid: char.normal_grid,
    adv_grid: char.adv_grid,
  };
  profile.keyed_upgrades = grids_to_keyed(char.normal_grid, char.adv_grid, {}, char.tier);
  for (const label of char.disabled ?? []) profile.bound_budgets.enabled[label] = false;
  return profile;
}

describe("export", () => {
  it("real", () => {
    setActivePinia(createPinia());
    const chosen = picks();
    const rng = xoroshiro128plus(SEED);
    const order = chosen.map((_, at) => at);
    for (let at = order.length - 1; at > 0; at--) {
      const other = uniformInt(rng, 0, at);
      [order[at], order[other]] = [order[other], order[at]];
    }
    const flipped = new Set(order.slice(0, Math.floor(order.length / 2)));
    for (const [at, { snapshot, char_id, first }] of chosen.entries()) {
      const profile = profile_of(snapshot, char_id);
      if (flipped.has(at)) profile.min_resolution = 1;
      const base = plain(job_for(profile, snapshot, true).payload);
      if (!base.upgrade_info.length) continue;
      const stem = `${snapshot.roster_id.slice(0, 8)}_${char_id.slice(0, 6)}`;
      const meta = {
        source: "analytics",
        at: new Date(snapshot.at).toISOString().slice(0, 10),
        tier: profile.tier,
        upgrades: base.upgrade_info.length,
        min_resolution: profile.min_resolution,
      };
      write("real", `real_${stem}`, { steps: [base], meta });
      if (!first) continue;

      // the weeks ahead at the Auto income of the character's item level
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
      const steps = [base];
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
      write("real_income", `realinc_${stem}`, {
        steps,
        meta: { ...meta, item_level, labels: ["now", ...steps.slice(1).map((_, w) => `+${w + 1}w`)] },
      });
    }
  });
});
