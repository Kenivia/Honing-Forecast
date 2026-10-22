import { check, done, equal, section } from "./helpers";
// Income from snapshots: reset times, the fit, what a snapshot holds per band, and a reload.
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { fit, owned_at, resets, value_of, WEEK_MS } from "@/Utils/Income";
import { snapshot_of, take_snapshot } from "@/Utils/Scans";
import { set_cell } from "@/Utils/InputColumn";
import { load_roster_config, write_state } from "@/Stores/ConfigStorage";

setActivePinia(createPinia());
const store = useRosterStore();
const config = store.roster_config;

section("resets");
const weeks = resets(Date.UTC(2026, 9, 19), Date.UTC(2026, 10, 3));
equal(
  "Thursdays 10:00 UTC",
  weeks.map((at) => new Date(at).toISOString()),
  ["2026-10-22T10:00:00.000Z", "2026-10-29T10:00:00.000Z"],
);

section("fit");
const slope = fit([
  { x: 0, y: 10 },
  { x: WEEK_MS, y: 30 },
  { x: 2 * WEEK_MS, y: 50 },
]);
check("slope per week", Math.abs(slope * WEEK_MS - 20) < 1e-9);
check("one point is no line", fit([{ x: 0, y: 1 }]) === null);
// a spend between the runs: one slope, each run its own intercept
const runs = fit([
  { x: 0, y: 100 },
  { x: WEEK_MS, y: 120 },
  { x: 2 * WEEK_MS, y: 10, breaks: true },
  { x: 3 * WEEK_MS, y: 30 },
]);
check("one slope over two runs", Math.abs(runs * WEEK_MS - 20) < 1e-9);
check(
  "a lone point adds no slope",
  fit([
    { x: 0, y: 0 },
    { x: WEEK_MS, y: 5 },
    { x: 2 * WEEK_MS, y: 999, breaks: true },
  ]) *
    WEEK_MS ===
    5,
);

section("a Serca character's snapshot");
const profile = config.profiles[0];
profile.tier = 1;
set_cell(profile.bound_budgets, "Fusion", "12");
set_cell(profile.bound_budgets, "Serca Fusion", "3");
set_cell(config.tradable_mats_owned[profile.roster_id], "Fusion", "10");
set_cell(profile.special_budget, "Serca Special Leap", "7");
const owned = owned_at(snapshot_of(config, profile.roster_id, "Manual"), profile.id, 1);
equal("T4 converted, tradable into roster", owned["Serca Fusion"], [5, 2, 0]);
equal("one band", value_of(owned, "Serca Fusion", 1), 2);
equal("free taps are char bound", owned["Serca Special Leap"], [7, 0, 0]);

section("snapshots survive a reload");
take_snapshot(config, profile.roster_id, "Manual");
config.snapshots[0].ignored = true;
profile.income_manual = true;
profile.bound_income["Serca Fusion"] = 4;
profile.income_now = { budgets: "abc", at: 5 };
profile.income_results = { abc: { gains: {}, gold: -1, leftover: -2 } };
config.roster_income[profile.roster_id] = [{ Gold: 100 }, {}];
for (let i = 0; i < 2; i++) {
  write_state(store);
  store.replace_config(load_roster_config());
}
const loaded = store.roster_config;
equal("snapshot", loaded.snapshots.length, 1);
check("prices kept", Object.keys(loaded.snapshots[0].prices).length > 0);
check("ignored kept", loaded.snapshots[0].ignored === true);
equal("char income", loaded.profiles[0].bound_income, { "Serca Fusion": 4 });
check("manual kept", loaded.profiles[0].income_manual);
equal("now pinned", loaded.profiles[0].income_now, { budgets: "abc", at: 5 });
equal("now's runs", loaded.profiles[0].income_results.abc.gold, -1);
equal("roster income", loaded.roster_income[profile.roster_id], [{ Gold: 100 }, {}]);

done();
