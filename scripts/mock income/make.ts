import fs from "node:fs";
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { export_config, import_config } from "@/Stores/ConfigStorage";
import { set_cell } from "@/Utils/InputColumn";
import { WEEK_MS } from "@/Utils/Income";
import { snapshot_of, type Snapshot } from "@/Utils/Scans";

// The real save with five weeks of made-up snapshots for roster 0, each material rising linearly
// in time, a spend (and a new line) at the fourth export, and an ignored outlier after the last.
// Toneema's char bound, roster 0's roster bound and tradable, per week.
const RATES = {
  bound: {
    Gold: 60_000,
    "Serca Red": 20_000,
    "Serca Blue": 50_000,
    "Serca Leaps": 1_500,
    "Serca Fusion": 300,
  },
  special: { "Serca Special Leap": 20 },
  roster: { Silver: 2_000_000 },
  tradable: { Gold: 40_000, "Serca Fusion": 100 },
};
const BASE = {
  bound: {
    Gold: 500_000,
    "Serca Red": 128_388,
    "Serca Blue": 369_213,
    "Serca Leaps": 6_162,
    "Serca Fusion": 1_475,
  },
  special: { "Serca Special Leap": 50 },
  roster: { Silver: 30_000_000 },
  tradable: { Gold: 200_000, "Serca Fusion": 400 },
};
const SPEND = {
  bound: { Gold: 300_000, "Serca Red": 100_000, "Serca Blue": 250_000 },
  special: {},
  roster: { Silver: 10_000_000 },
  tradable: { Gold: 150_000 },
};
const SPENT_CHEST = "Honing Leapstone Selection Chest VIII";

const [source, out] = process.argv.slice(2);
const FIRST = Date.UTC(2026, 8, 18, 18);
const SPENT_AT = FIRST + 3 * WEEK_MS;
const NOW = Date.UTC(2026, 9, 21, 12);

setActivePinia(createPinia());
const store = useRosterStore();
store.replace_config(import_config(fs.readFileSync(source, "utf8")));
const config = store.roster_config;
const profile = config.profiles.find((p) => p.char_name === "Toneema");
config.active_profile_index = config.profiles.indexOf(profile);

// the budgets as they are at `t`, an outlier multiplying Gold
function set_budgets(t: number, outlier = 1) {
  const weeks = (t - FIRST) / WEEK_MS;
  const spent = t >= SPENT_AT;
  const columns = {
    bound: profile.bound_budgets,
    special: profile.special_budget,
    roster: config.roster_mats_owned[0],
    tradable: config.tradable_mats_owned[0],
  };
  for (const [band, column] of Object.entries(columns)) {
    for (const [label, base] of Object.entries(BASE[band])) {
      let amount =
        (base as number) +
        RATES[band][label] * weeks -
        (spent ? (SPEND[band][label] ?? 0) : 0);
      if (label === "Gold" && band === "bound") amount *= outlier;
      set_cell(column, label, String(Math.round(amount)));
    }
  }
  const chest = profile.chests.find((c) => c.title === SPENT_CHEST);
  chest.count = spent ? 100 : 140;
}
// a snapshot at `t` of the budgets as they are at `budgets_at`
function add(
  t: number,
  kind: Snapshot["kind"],
  extra: Partial<Snapshot> = {},
  budgets_at = t,
  outlier = 1,
) {
  set_budgets(budgets_at, outlier);
  config.snapshots.push({ ...snapshot_of(config, 0, kind), at: t, ...extra });
}

config.snapshots = [];
for (let week = 0; week < 5; week++) {
  const at = FIRST + week * WEEK_MS;
  // before an export the budgets are as the last export left them
  add(at, "Before export", {}, at - WEEK_MS);
  add(at + 120_000, "After export", week === 3 ? { breaks: true } : {});
  if (week === 2) add(at + WEEK_MS / 2, "Manual");
  if (week === 4) add(at + WEEK_MS / 2, "Manual", { ignored: true }, at, 5);
}
set_budgets(NOW);
fs.writeFileSync(out, export_config(config) + "\n");
console.log(`${config.snapshots.length} snapshots written to ${out}`);
