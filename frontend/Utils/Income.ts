import { ALL_LABELS, NUM_BANDS, SPECIAL_LEAP_LABELS } from "./Constants";
import { chest_gains, process_chests } from "./Chest";
import type { Snapshot, SnapshotResult } from "./Scans";
import { merge_lower_tier } from "@/WasmInterface/PayloadBuilder";

// Income per week from budget snapshots: what one character has per band at each snapshot, and a
// least squares line through it.

export const WEEK_MS = 7 * 24 * 3600 * 1000;
// a weekly reset, Thursday 10:00 UTC; both regions reset together
const RESET = Date.UTC(2026, 0, 1, 10);
// the least past and future the graph shows
export const MIN_SPAN = 5 * WEEK_MS;

export function resets(from: number, to: number) {
  const out: number[] = [];
  for (
    let at = RESET + Math.ceil((from - RESET) / WEEK_MS) * WEEK_MS;
    at <= to;
    at += WEEK_MS
  )
    out.push(at);
  return out;
}

// per label of the tier, what a character owns per band, the lower tier converted up, plus what
// the chests give; free taps under the special leap label, all char bound
export type Owned = Record<string, number[]>;

export function char_chests(snapshot: Snapshot, char_id: string) {
  return [
    ...(snapshot.chars[char_id]?.chests ?? []),
    ...snapshot.roster_chests,
  ];
}

export function owned_at(
  snapshot: Snapshot,
  char_id: string,
  tier: number,
  result?: SnapshotResult,
): Owned {
  const char = snapshot.chars[char_id];
  const merged = merge_lower_tier(
    tier,
    char?.bound ?? {},
    snapshot.roster_owned,
    snapshot.tradable,
  );
  // without an optimizer run, every chest on its first option as Rust does
  const gains =
    result?.gains ??
    chest_gains(process_chests(char_chests(snapshot, char_id), tier));
  const out: Owned = {};
  for (const label of ALL_LABELS[tier]) {
    out[label] = [merged.bound, merged.roster, merged.tradable].map(
      (column, band) => (column[label] ?? 0) + (gains[label]?.[band] ?? 0),
    );
  }
  const special = SPECIAL_LEAP_LABELS[tier];
  out[special] = [
    (merge_lower_tier(tier, char?.special ?? {}, {}, {}).bound[special] ?? 0) +
      (gains[special] ?? []).reduce((a, b) => a + b, 0),
    0,
    0,
  ];
  return out;
}

// one band, or the bands up to it summed
export function value_of(
  owned: Owned,
  label: string,
  band: number,
  cumulative: boolean,
) {
  const bands = owned[label] ?? Array(NUM_BANDS).fill(0);
  return cumulative
    ? bands.slice(0, band + 1).reduce((a, b) => a + b, 0)
    : bands[band];
}

export interface Point {
  x: number;
  y: number;
  // starts a new line
  breaks?: boolean;
}

// One least squares slope through runs of points, each run with its own intercept. Null without
// two distinct times in a run.
export function fit(points: Point[]) {
  const runs: Point[][] = [];
  for (const point of points) {
    if (!runs.length || point.breaks) runs.push([]);
    runs[runs.length - 1].push(point);
  }
  let sxx = 0;
  let sxy = 0;
  const means = runs.map((run) => {
    const mx = run.reduce((a, p) => a + p.x, 0) / run.length;
    const my = run.reduce((a, p) => a + p.y, 0) / run.length;
    for (const p of run) {
      sxx += (p.x - mx) ** 2;
      sxy += (p.x - mx) * (p.y - my);
    }
    return { mx, my };
  });
  if (!sxx) return null;
  const slope = sxy / sxx;
  return {
    slope,
    runs: runs.map((run, at) => ({
      from: run[0].x,
      to: run[run.length - 1].x,
      at: (x: number) => means[at].my + slope * (x - means[at].mx),
    })),
  };
}
